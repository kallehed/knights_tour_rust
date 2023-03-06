#![allow(unused_variables, dead_code, unused_imports)]
#![allow(clippy::assertions_on_constants)] // prevents warnings about static asserts

// Code explanation:
//
// Think of a grid like this:
// 
//          j           
//
//       012345
//      0
//      1
//  i   2
//      3
//      4
//      5
//      .
//      .
//      .
//
// , where a horse can start on any cell and jump where it wants to.
// it may jump to absurdly high i's, and that we call going to infinity.
// It is bounded by the width, which in this case is 6. 
// Other facts: The horse may not jump to where it has been before.
// It has an order of ways to jump. 

use std::slice::SliceIndex;

/// type used for indexing into the horse matrix
type Int = i32;

const MAX_HEIGHT: usize = 128;
/// Where to stop and declare -> TO INFINITY!
const TOP: Int = (MAX_HEIGHT - 3) as Int; 
/// at what range to start researching widths, 3 normally
const START_WIDTH: usize = 1;
/// maximum
const MAX_WIDTH: usize = 42;

/// The stack size each thread gets, increase if stack-overflow
const STACK_SIZE: usize = (MAX_WIDTH * MAX_HEIGHT + 1).next_power_of_two() + 1;

// static assert
const _: () = assert!(MAX_WIDTH >= START_WIDTH);

fn main()
{
    println!("Hello, world!");
    println!("stack size: {}", STACK_SIZE);
    //println!("available parallelism: {}", std::thread::available_parallelism().unwrap());

    use std::time::{Duration, Instant};

    let start = Instant::now();

    //print_positions_not_halt();
    print_all_inf_jump_seq_patterns();
    if false {
        //let a = jump_seq(10, 25, 42);
        let a = jump_seq(30, 0, 3);

        println!("{:?}", a);

        println!("found pattern: {:?}", pattern(&a));
    }

    let duration = start.elapsed();

    println!("Took: {:?}", duration);

    println!("End");
}

// 23/20/22 sec from 1 to 500 with (width) many threads

// { 22.6 from 1 to 512 with threads and atomic
// { 211.7 from 1 to 1024 wtih threads and atomic

// 26 from 1 to 512 with unsigned numbers instead of signed, 13% slower than with signed

/// for all the patterns to infinity found -> compute their pattern details(starts_at, repeats_in...) 
fn print_all_inf_jump_seq_patterns()
{
    use std::sync::Mutex;

    #[derive(Debug)]
    struct Data {
        i: Int,
        j: Int,
        width: Int,
        pat: JumpPattern
    }
    let data = Mutex::new(Vec::new());


    positions_not_halt_giver(|width, i, j|
    {
        let seq = jump_seq(i, j, width); 
        let pat = pattern(&seq);
        //println!("Width: {}, at x={}, y={}, Pattern: {:?} rep/width: {}", width, j, i, pat, div);
        {
            data.lock().unwrap().push(Data{i, j, width, pat});
        }
    });

    let mut data = data.into_inner().unwrap();
    data.sort_by(|x, y| { // sorts by width first, then i
        let a = x.width.cmp(&y.width);
        match a {
            std::cmp::Ordering::Equal =>
            {
                x.i.cmp(&y.i)
            }
            _ => a
        }
    });
    //data.sort_by(|x, y| {x.pat.starts_at.cmp(&y.pat.starts_at)});
    for dat in data {
        let div = dat.pat.repeats_in as f64/dat.width as f64;
        println!("Width: {}, i:{}, j:{}, Pattern: {:?}, rep/width: {}", dat.width, dat.i, dat.j, dat.pat, div);
    }
}

fn print_positions_not_halt()
{
    positions_not_halt_giver(|width, i, j|
    {
        println!("Width: {}, at x={}, y={}", width, j, i);
    });
}

/// giver gives width, i and j. 
fn positions_not_halt_giver<F>(giver: F) where F: Fn(Int, Int, Int) + Send + Sync
{
    use std::thread;

    // set number of threads to the most optimal
    let threads = thread::available_parallelism()
                                .unwrap_or(std::num::NonZeroUsize::new(8).unwrap());
    //println!("widths per thread: {}", widths_per_thread);
    use std::sync::atomic::{AtomicI32, Ordering};

    // the width we are at computing
    let width_at = AtomicI32::new(START_WIDTH as _);
    thread::scope(|s| {
        for thread in 0..threads.into() {
            let width_at = &width_at;
            let giver = &giver;
            thread::Builder::new()
                .stack_size(STACK_SIZE)
                .spawn_scoped(s, move ||
            {
                //println!("thread num: {}", thread);
                
                loop {
                    // try to get work to do
                    let width = width_at.fetch_add(1, Ordering::Relaxed);
                        
                        //println!("width at: {}", width);
                    if width > MAX_WIDTH as _ {
                        return;
                    } 
                    
                    for j in 0..width {
                        for i in 0..TOP {
                            let res = halts(i as _, j as _, width as _);
            
                            if !res {
                                giver(width, i, j);
                            }
                        }
                    }
                    
                }
                    
            }).unwrap();
        }
    });
}

#[allow(clippy::never_loop)] // clippy erroneously gives this warning here
/// Returns whether, from a starting point and width, the horse halts or not
fn halts(start_i: Int, start_j: Int, width: Int) -> bool
{
    assert!(width <= MAX_WIDTH as _);
    let mut matrix = [[false; MAX_WIDTH]; MAX_HEIGHT];

    let mut i: Int = start_i;
    let mut j: Int = start_j;

    loop {
        matrix[i as usize][j as usize] = true;
        //unsafe {*matrix.get_unchecked_mut(i as usize).get_unchecked_mut(j as usize) = true;}

        const JUMPS: [(i32,i32); 8] = [(-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)];

        'good: {
            for jump in JUMPS
            {
                let i_t = i + jump.0; // temporaries
                let j_t = j + jump.1;
                if i_t >= 0 && j_t >= 0 && j_t < width &&
                    !matrix[i_t as usize][j_t as usize]
                {
                    //unsafe {!matrix.get_unchecked(i_t as usize).get_unchecked(j_t as usize)}
                        // valid jump
                    i = i_t;
                    j = j_t;
                    break 'good;
                }
            }
            return true // if no jumps available, exit
        }
        if i > TOP {
            return false; // does not halt, as it reached the top, infinitely high
        }
    }}

type JumpNum = u8;

#[allow(clippy::never_loop)]
/// a function that returns the numbers 0..8 representing horse jumps, at a certain width and start place
fn jump_seq(start_i: Int, start_j: Int, width: Int) -> Vec<JumpNum>
{
    assert!(width <= MAX_WIDTH as _);
    let mut matrix = [[false; MAX_WIDTH]; MAX_HEIGHT];

    let mut jumps = Vec::new();

    let mut i: Int = start_i;
    let mut j: Int = start_j;

    loop {
        matrix[i as usize][j as usize] = true;
        //unsafe {*matrix.get_unchecked_mut(i as usize).get_unchecked_mut(j as usize) = true;}

        const JUMPS: [(i32,i32); 8] = [(-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)];

        'good: {
            for (idx, jump) in JUMPS.into_iter().enumerate()
            {
                let i_t = i + jump.0; // temporaries
                let j_t = j + jump.1;
                if i_t >= 0 && j_t >= 0 && j_t < width &&
                    !matrix[i_t as usize][j_t as usize]
                {
                    //unsafe {!matrix.get_unchecked(i_t as usize).get_unchecked(j_t as usize)}
                        // valid jump
                    i = i_t;
                    j = j_t;
                    jumps.push(idx as JumpNum);
                    break 'good;
                }
            }
            return jumps // if no jumps available, exit
        }
        if i > TOP {
            return jumps;
        }
    }

}

#[derive(Debug, Copy, Clone)]
struct JumpPattern {
    starts_at: usize,
    repeats_in: usize,
}

/// Takes in numbers from 0..8 and returns the largest pattern found
fn pattern(jumps: &[JumpNum]) -> JumpPattern
{
    let len = jumps.len();
    for i in 0..len {
        for step in 1..((len - i)/2) {
            let start = &jumps[i..(i+step)];
            for j in ((i+step)..).step_by(step) {
                if j + step >= len {
                    //println!("pattern found: i: {} step: {}, j: {}, len: {}", i, step, j, len);
                    return JumpPattern{starts_at: i, repeats_in: step};
                }
                if start != &jumps[j..(j+step)]
                {
                    break;
                }
            }
        }
    }
    unreachable!() // should probably be unreachable, I think?
}