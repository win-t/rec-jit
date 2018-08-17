//! Based on https://nullprogram.com/blog/2015/03/19/
//! C version from the author: https://gist.github.com/skeeto/3a1aa3df31896c9956dc

// need to create a module, so private things cannot be accessed from ouside.
mod asm_buf {

    // hardcoded for now,
    // if total opcode size bigger than this, it will panic
    pub const BUF_SIZE: usize = 4096;

    use std::io::{Cursor, Write};
    use std::mem::transmute;
    use std::slice::from_raw_parts_mut;

    extern crate libc;
    use self::libc::{
        mmap, mprotect, munmap, c_void,
        PROT_EXEC, PROT_READ, PROT_WRITE,
        MAP_ANONYMOUS, MAP_PRIVATE, MAP_FAILED,
    };

    pub struct AsmBuf {
        locked: bool,
        cursor: Cursor<&'static mut [u8]>,
    }

    pub fn new() -> AsmBuf {
        let prot = PROT_READ | PROT_WRITE;
        let flags = MAP_ANONYMOUS | MAP_PRIVATE;

        let begin = unsafe { mmap(0 as *mut c_void, BUF_SIZE, prot, flags, -1, 0) };
        assert_ne!(begin, MAP_FAILED, "Failed to mmap");
        let data = unsafe { from_raw_parts_mut(begin as *mut u8, BUF_SIZE) };

        let mut retval = AsmBuf {
            locked: false,
            cursor: Cursor::new(data),
        };

        retval.op_mov_rax_rdi();

        retval
    }

    impl Drop for AsmBuf {
        fn drop(&mut self) {
            let data = self.cursor.get_mut();
            let data_ptr = data.as_mut_ptr() as *mut c_void;
            let data_len = data.len();
            let retval = unsafe { munmap(data_ptr, data_len) };
            assert_ne!(retval, -1, "Failed to munmap");
            // basically, self.cursor is dangling pointer now
        }
    }

    impl AsmBuf {
        fn write(&mut self, buf: &[u8]) {
            assert!(!self.locked, "AsmBuf are locked");
            self.cursor.write_all(buf)
                .expect("Failed to write, buffer overflow");
        }

        pub fn to_fn<'a>(&'a mut self) -> impl Fn(i64) -> i64 + 'a {
            if !self.locked {
                self.op_ret();

                self.locked = true; // no write after this

                let data = self.cursor.get_mut();
                let data_ptr = data.as_mut_ptr() as *mut c_void;
                let data_len = data.len();
                let retval = unsafe { mprotect(data_ptr, data_len, PROT_READ | PROT_EXEC) };
                assert_ne!(retval, -1, "Failed to mprotect");
            }
            unsafe {
                transmute::<_, fn(i64) -> i64>(self.cursor.get_mut().as_mut_ptr() as *mut c_void)
            }
        }
    }

    macro_rules! generate_op {
        ($name:ident, $data:expr) => {
            pub fn $name(&mut self) -> &mut Self {
                self.write($data);
                self
            }
        };
    }

    impl AsmBuf{
        // only some (safe) opcode allowed here,
        // if we let arbitary opcode,
        // then calling to fn returned by to_fn should be considered unsafe

        generate_op!(op_add_rax_rdi, &[0x48, 0x01, 0xF8]);
        generate_op!(op_and_rax_rdi, &[0x48, 0x21, 0xF8]);
        generate_op!(op_idiv_rdi, &[0x48, 0xF7, 0xFF]);
        generate_op!(op_imul_rax_rdi, &[0x48, 0x0F, 0xAF, 0xC7]);
        generate_op!(op_mov_ecx_edi, &[0x89, 0xF9]);
        generate_op!(op_mov_rax_rdi, &[0x48, 0x89, 0xF8]);
        generate_op!(op_mov_rax_rdx, &[0x48, 0x89, 0xD0]);
        generate_op!(op_not_rax, &[0x48, 0xF7 ,0xD0]);
        generate_op!(op_or_rax_rdi, &[0x48, 0x09, 0xF8]);
        generate_op!(op_ret, &[0xC3]);
        generate_op!(op_shl_rax_cl, &[0x48, 0xD3, 0xE0]);
        generate_op!(op_shr_rax_cl, &[0x48, 0xD3, 0xE8]);
        generate_op!(op_sub_rax_rdi, &[0x48, 0x29, 0xF8]);
        generate_op!(op_xor_rax_rdi, &[0x48, 0x31, 0xF8]);
        generate_op!(op_xor_rdx_rdx, &[0x48, 0x31, 0xD2]);

        pub fn op_mov_rdi_imm64(&mut self, imm: i64) -> &mut Self {
            self.write(&[0x48, 0xBF]);
            self.write(&unsafe { transmute::<i64, [u8; 8]>(imm) });
            self
        }
    }
}


use std::iter::Peekable;
use std::io::Read;

fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let mut input = buffer.bytes().peekable();

    let mut asmbuf = asm_buf::new();

    while let Some(c) = input.next() {
        if c == b'\n' {
            break;
        }

        if c == b' ' {
            continue;
        }

        let operator = c;

        // not operator (!)

        if operator == b'!' {
            asmbuf.op_not_rax();
            continue; // not parsing the digit
        }

        // parse digit
        let operand = parse_digit(&mut input);
        asmbuf.op_mov_rdi_imm64(operand);

        match operator {
            b'+' => { asmbuf.op_add_rax_rdi(); },
            b'-' => { asmbuf.op_sub_rax_rdi(); },
            b'*' => { asmbuf.op_imul_rax_rdi(); },
            b'/' => { asmbuf.op_xor_rdx_rdx()
                            .op_idiv_rdi(); },
            // Exercise: implements
            // - mod (%)
            // - xor (^)
            // - shl (<)
            // - shr (>)
            b'%' => { asmbuf.op_xor_rdx_rdx()
                            .op_idiv_rdi()
                            .op_mov_rax_rdx(); },
            b'^' => { asmbuf.op_xor_rax_rdi(); },
            b'<' => { asmbuf.op_mov_ecx_edi()
                            .op_shl_rax_cl(); },
            b'>' => { asmbuf.op_mov_ecx_edi()
                            .op_shr_rax_cl(); },
            // Extra: instruction
            // - and (&)
            // - or (|)
            // - not (!) [not operand, written above]
            b'&' => { asmbuf.op_and_rax_rdi(); },
            b'|' => { asmbuf.op_or_rax_rdi(); },
            _ => panic!("unkonwn operator"),
        }
    }

    let fun = asmbuf.to_fn();

    // try to uncoment this
    // it should compile error, because fun cannot live longer than asmbuf
    // drop(asmbuf);

    let init = parse_digit(&mut input);
    let term = parse_digit(&mut input);
    let mut x = init;
    for i in 0..term+1 {
        println!("Term {}: {}", i, x);
        x = fun(x);
    }
}

fn parse_digit(input: &mut Peekable<impl Iterator<Item=u8>>) -> i64 {
    let mut num = 0;
    while let Some(&c) = input.peek() {
        if c != b' ' && c != b'\n' {
            break;
        }
        input.next();
    }
    let sign = if let Some(&b'-') = input.peek() {
        input.next();
        -1
    } else {
        1
    };

    while let Some(&c) = input.peek() {
        if c < b'0' || c > b'9' {
            break;
        }
        let digit = (c - b'0') as i64;
        num = num * 10 + digit;
        input.next();
    }
    sign * num
}
