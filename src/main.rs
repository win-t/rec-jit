/// Based on https://nullprogram.com/blog/2015/03/19/
/// C version from the author: https://gist.github.com/skeeto/3a1aa3df31896c9956dc
extern crate libc;

use std::ptr;
use std::mem;
use std::iter::Peekable;
use libc::{
    mmap, mprotect, munmap, c_void,
    PROT_EXEC, PROT_READ, PROT_WRITE,
    MAP_ANONYMOUS, MAP_PRIVATE,
};

const PAGE_SIZE: usize = 4096;

struct AsmBuf {
    begin: *mut u8,
    cursor: *mut u8,
}

impl AsmBuf {
    fn new() -> AsmBuf {
        let prot = PROT_READ | PROT_WRITE;
        let flags = MAP_ANONYMOUS | MAP_PRIVATE;
        let null = ptr::null_mut();

        let begin = unsafe { mmap(null, PAGE_SIZE, prot, flags, -1, 0) } as *mut u8;
        AsmBuf {
            begin: begin,
            cursor: begin,
        }
    }
    fn ins(&mut self, size: usize, ins: usize) {
        for i in (0..size).rev() {
            unsafe {
                let val = (ins >> (i * 8)) as u8;
                self.cursor.write(val);
                self.cursor = self.cursor.add(1);
            }
        }
    }
    fn immediate(&mut self, size: usize, value: i64) {
        unsafe {
            let value = &value as *const i64 as *const u8;
            self.cursor.copy_from_nonoverlapping(value, size);
            self.cursor = self.cursor.add(size);
        }
    }
    fn to_fn(&mut self) -> fn(i64) -> i64 {
        unsafe {
            mprotect(self.begin as *mut c_void, PAGE_SIZE, PROT_READ | PROT_EXEC);
            mem::transmute(self.begin)
        }
    }
}

impl Drop for AsmBuf {
    fn drop(&mut self) {
        unsafe {
            munmap(self.begin as *mut c_void, PAGE_SIZE);
        }
    }
}

fn main() {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    let mut input = buffer.bytes().peekable();

    let mut asmbuf = AsmBuf::new();

    asmbuf.ins(3, 0x4889F8); // mov rax, rdi

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
            asmbuf.ins(3, 0x48F7D0); // not rax
            continue; // not parsing the digit
        }

        // parse digit
        let operand = parse_digit(&mut input);

        asmbuf.ins(2, 0x48BF); // mov rdi, operand
        asmbuf.immediate(8, operand);

        match operator {
            b'+' => asmbuf.ins(3, 0x4801F8),   // add rax, rdi
            b'-' => asmbuf.ins(3, 0x4829F8),   // sub rax, rdi
            b'*' => asmbuf.ins(4, 0x480FAFC7), // imul rax, rdi
            b'/' => {
                    asmbuf.ins(3, 0x4831D2);   // xor rdx, rdx
                    asmbuf.ins(3, 0x48F7FF);   // idiv rdi
            },
            // Exercise: implements
            // - mod (%)
            // - xor (^)
            // - shl (<)
            // - shr (>)
            b'%' => {
                    asmbuf.ins(3, 0x4831D2);   // xor rdx, rdx
                    asmbuf.ins(3, 0x48F7FF);   // idiv rdi
                    asmbuf.ins(3, 0x4889D0);   // mov rax, rdx
            },
            b'^' => asmbuf.ins(3, 0x4831F8),   // xor rax, rdi
            b'<' => {
                    asmbuf.ins(2, 0x89F9);     // mov ecx, edi
                    asmbuf.ins(3, 0x48D3E0);   // shl rax, cl
            },
            b'>' => {
                    asmbuf.ins(2, 0x89F9);     // mov ecx, edi
                    asmbuf.ins(3, 0x48D3E8);   // shr rax, cl
            },
            // Extra: instruction
            // - and (&)
            // - or (|)
            // - not (!) [no operand, written above]
            b'&' => asmbuf.ins(3, 0x4821F8),   // and rax, rdi
            b'|' => asmbuf.ins(3, 0x4809F8),   // or rax, rdi
            _ => panic!("unkonwn operator"),
        }
    }

    asmbuf.ins(1, 0xC3); // ret
    let fun = asmbuf.to_fn();

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
