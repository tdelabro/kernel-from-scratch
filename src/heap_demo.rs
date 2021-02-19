//! Heap demo scenarii 
//!
//! Temporary file, will be used during the project correction at 42.

#![allow(unused_imports)]
#![allow(dead_code)]

use dynamic_memory_management::{KERNEL_HEAP, Heap, Locked};
use core::alloc::{Layout, GlobalAlloc, Allocator};
use core::ptr::{NonNull};
use physical_memory_management::{BITMAP};
use alloc::boxed::Box;

fn box_demo() {
    {
        println!("");
        print!("{}", KERNEL_HEAP.lock());
        println!("-----");
        let a = Box::new(1usize);
        println!("a at {:p}, value: {}", a, a);
        print!("{}", KERNEL_HEAP.lock());
        println!("-----");
        let b = Box::new(2usize);
        let c = Box::new(3usize);
        drop(b);
        println!("a at {:p}, value: {}", a, a);
        println!("c at {:p}, value: {}", c, c);
        print!("{}", KERNEL_HEAP.lock());
        println!("-----");
    }
    println!("Out of block, everything had been droped");
    print!("{}", KERNEL_HEAP.lock());
    println!("-----");
}

fn expand_heap_demo() {
    print!("{}", KERNEL_HEAP.lock());
    println!("-----");
    {
        let _big = Box::new([0u32; 1025]);
        print!("{}", KERNEL_HEAP.lock());
        println!("-----");
    }
    println!("Out of block, everything had been droped");
    print!("{}", KERNEL_HEAP.lock());
    println!("-----");
}

// Compiler crash no good
fn local_heap_demo() {
    let my_allocator = unsafe { Locked::new(Heap::new(0xc00000 as *const usize, false)) };
    let a = Box::new_in(42usize, my_allocator);
    println!("{}", a);
}

fn allocator_method_demo() {
    let my_allocator = unsafe { Locked::new(Heap::new(0x7FFF000 as *const usize, false)) };
    let mut x = my_allocator.allocate(Layout::new::<usize>()).unwrap().cast::<usize>();
    unsafe {
        *x.as_mut() = 4;
        println!("{}", x.as_ref());
    }
    print!("{}", my_allocator.lock());
    println!("-----");
    unsafe { my_allocator.deallocate(x.cast::<u8>(), Layout::new::<usize>()) };
    print!("{}", my_allocator.lock());
    println!("-----");
    //println!("{}", BITMAP.lock());
}

fn infinite_alloc_demo() {
    let my_allocator = unsafe { Locked::new(Heap::new(0x7FFE000 as *const usize, false)) };
    let mut c = 0;
    while let Ok(x) = my_allocator.allocate(Layout::new::<[u32; 1024]>()) {
        println!("{} {:p} {}", c, x, my_allocator.lock());
        c += 1; 
    }
    println!("{}", BITMAP.lock());
}

pub fn demo() {
    //box_demo();
    //expand_heap_demo();
    //allocator_method_demo();
    infinite_alloc_demo();
}
