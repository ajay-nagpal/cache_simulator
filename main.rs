use std::env;                        //read command line arguments
use getopts::Options;               //Cli arguments parsing

use std::fs::File;                // file handling
use std::io::{BufReader,BufRead};// buffered reading of trace file


/*
Structure to represent a cache lines
*/
struct Line{
  contain_block:bool,
  tag:u64,
  last_used:u64,
}

/*
Structure to represent a cache set
each set contains multiple cache lines
*/
struct Set{
  lines:Vec<Line>,
}

/*
Structure representing the cache
stores various cache parameters
*/
struct Cache{
  sets:Vec<Set>,//collection of cache set
  s:usize,     // number of set index bits
  b:usize,    // number of block offset bits

  hits:u64,
  miss:u64,
  evicts:u64,

  global_counter:u64,//global counter for LRU replacement
}

/*
Enums HIT , MISS and FULL
representing the cache lookup result
*/
#[derive(PartialEq)] // attribute to allow enum values to be comared using == or !=
enum SearchResult{
  HIT,
  MISS,
  FULL,
}

/*
Function to print invalid format message when entered  cli is invalid
*/
fn print_msg(){
  println!("invalid cli!");
  println!("required flags: -s <s> -E <E> -b <b> -t <tracefile>");
}


/*
is_full is used for checking if there is any line in a 
particular set is available for storing a block logically.
if for any line in a particular set, contain_block is false
then line in that set is free otherwise the set is considered full.
*/
fn is_full(set_index:usize,cache:&mut Cache)->bool{
  //use set index to determine set
  //search for free line in that set

  let set:& Set=& cache.sets[set_index];
  
  for line in &set.lines{
    if ! line.contain_block {
      return false;
    }
  }
  true 
}

/*
This function checks whether the memory address access results in 
a cache HIT, it iterate through the lines of a particular set selected using set_index parameter
and compared the logically placed tag with the tag of the requested address.
If a match is found and line contains the block , then the function returns true and
updates the LRU placement policy related information and increase the
cache's global_counter for the next use.
*/
fn is_hit(tag:u64,set_index:usize,cache:&mut Cache)->bool{
  //use set index to determine set
  //search in lines of that set

  let set:&mut Set=&mut cache.sets[set_index];
  let lines:&mut Vec<Line>=&mut set.lines;

  for line in lines{
    if line.contain_block && line.tag==tag{
      line.last_used=cache.global_counter;
      cache.global_counter+=1;
      return true;
    }
  }
  false 
}

/*
This function coordinates with is_hit and is_full functions to determine
whether an address lookup results in a hit or miss with available or
miss requiring eviction i.e full.
The function returns enums HIT, MISS, FULL to indicate the result.
*/
fn check_cache(tag:u64,set_index:usize,cache:&mut Cache)->SearchResult{
  if is_hit(tag,set_index,cache){
    return SearchResult::HIT;
  }
  if is_full(set_index,cache){
    return SearchResult::FULL;
  }
  SearchResult::MISS
}

/*
The function takes a memory address,number of set index bits s and
number of block offset bits b as arguments and calculate the 
set index and tag using bit masks and bit operations.

*/
fn parse_address(address:u64, s:usize, b:usize)->(u64,usize){
  //remove the block offset bits and calculate shifted address
  let shifted_address=address>>b;
  
  //mask for the set index: s lower bits set to 1
  let set_index_mask=(1<<s)-1;

  //extract the set index(next s bits)
  let set_index=(shifted_address & set_index_mask) as usize;

  //everything remaining after shifting by s is the tag
  let tag=shifted_address>>s;

  (tag,set_index)
}

/*
operate_cache function take an address, a cache and a instruction counter to 
count how many times a address lookup for that instruction should happen in a cache.
this function uses the check_cache function to determine  whether the address is in the cache.

if the address is found the function increment the hit count,
if the address is not found and an empty spot exists in the set, 
then the function logically insert that block into the cache
to make it appear that the address is into the cache 
and increment the miss count.

if the address is not found and set is full, the function calls the evict function
to evict a block from a line from a particular set selected using set_index
and  increments the eviction count and the miss count.
Then the function logically insert that block into the cache
to make it appear that the address is inserted into the cache.
*/
fn operate_cache(address:u64, cache: & mut Cache, loop_counter:usize){
  let (tag,set_index)=parse_address(address,cache.s,cache.b);

  for _ in 0..loop_counter{
    let result:SearchResult=check_cache(tag,set_index,cache);

    if result==SearchResult::HIT{
      cache.hits+=1;
    }
    else if result==SearchResult::MISS{
      cache.miss+=1;
    }
    else{
      cache.miss+=1;
      cache.evicts+=1;
    }
  }
} 


/*
This function extract the memory address from a trace line.
expected trace format=>
operation address, size

function split the trace line into two parts
1. operation character 
2. address and size parts

address part is then separated from size parameter and converted 
from hexadecimal string format into  unsigned 64 bit integer.

This function returns Option<u64>  to avoid failure in process and 
if anything fails this function returns None so that the caller can
safely skip the incorrect trace lines.
*/
fn extract_address(line:&str)->Option<u64>{

  let parts:Vec<&str>=line.split_whitespace().collect();

  if parts.len()!=2{
    return None;
  }

  let adr_part:Vec<&str>=parts[1].split(',').collect();

  if adr_part.len()!=2{
    return None;
  }

  let adr_str:&str=adr_part[0];

  let address:u64=match u64::from_str_radix(adr_str,16){
    Ok(addr)=>addr,
    Err(_)=>return None,
  };
  
  Some(address)
}

/*
This function takes a trace file and a cache as a input.
reads the trace file line by line.
for every line:-
leading and trailing whitespaces removed to simplify parsing.
empty lines and lines starting with 'I'(Instruction load) are ignored.
The operation type(M|L|S) is checked to count  how many times the instruction needs cache lookup,
then accordingly the operation_cache_access_count value is set,

for data load and data store instruction it is one 
and for data modify instruction it is 2  because it represents 
a data load followed by a data store .

We skip the process for other fields because trace files may contain several additional fields.
we extract address from this address string using extract_address function.
*/
fn operate_flags(trace_file:&str,cache:&mut Cache){
  let file=match File::open(trace_file){
    Ok(file)=>file,
    Err(err)=>{
      println!("failed to open the file: {}",err);
      std::process::exit(1);
    }  
  };

  let reader=BufReader::new(file);

  for line in reader.lines(){
    let address_str=match line{
      Ok(v)=>v,
      Err(_)=>continue,
    };

    let trimmed_address_str=address_str.trim();

    if trimmed_address_str.starts_with('I') || trimmed_address_str.is_empty(){
      continue;
    }

    let operation_cache_access_count= 
    if trimmed_address_str.starts_with('M') {
      2
    } 
    else if trimmed_address_str.starts_with('L') || trimmed_address_str.starts_with('S'){
      1
    }
    else{
      continue;
    };

    let address:u64=match extract_address(&trimmed_address_str){
      Some(addr)=>addr,
      None=>continue,
    };
        
    operate_cache(address,cache,operation_cache_access_count);
  }
}


/*
This is the entry point of the simulator.
the program expect the arguments in the exact order specified in the coursework
Required Flags: -s <s> -E <E> -b <b> -t <tracefile> .

The getopts crate is then used to safely extract the flag values.
if the flag is incorrect, then the program prints help message
and terminates.
*/
pub fn main() {
  let args:Vec<String>=env::args().collect();
  if args.len()!=9{
    print_msg();
    return;
  }

  let mut opts=Options::new();

  opts.optopt("s","","","");
  opts.optopt("b","","","");
  opts.optopt("E","","","");
  opts.optopt("t","","","");

  let matches:getopts::Matches=match opts.parse(&args[1..]){
    Ok(m)=>m,
    Err(_f)=>{
      print_msg();
      return;
    }
  };

  if !matches.free.is_empty(){
    print_msg();
    return;
  }

  let s=matches.opt_str("s");
  let b=matches.opt_str("b");
  let e=matches.opt_str("E");
  let t=matches.opt_str("t");

  if s.is_none()||b.is_none()||e.is_none()||t.is_none(){
    print_msg();
    return;
  }

  let (s,e,b):(usize,usize,usize)=match(
    s.unwrap().parse(),
    e.unwrap().parse(),
    b.unwrap().parse()
  ){
    (Ok(sv),Ok(ev),Ok(bv))=>(sv,ev,bv),
    _ =>{
      print_msg();
      return;
    }
  };

  let trace_file:String=t.unwrap();

  let mut sets:Vec<Set>=Vec::new();
  let total_sets:usize=1<<s;

  for _ in 0..total_sets{
    let mut lines:Vec<Line>=Vec::new();

    for _ in 0..e{
      lines.push(Line{
        contain_block:false,
        tag:0,
        last_used:0,
      });
    }
    sets.push(Set{lines});
  }

  let mut cache=Cache{
    sets,
    s:s,
    b:b,
    // _e:e,
    hits:0,
    miss:0,
    evicts:0,

    global_counter:0,
  };

  operate_flags(trace_file.as_str(),&mut cache);

}
