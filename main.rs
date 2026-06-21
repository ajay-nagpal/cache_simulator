use std::env;                      //read command line arguments
use getopts::Options;             //Cli arguments parsing

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
Function to print invalid format message when entered  cli is invalid
*/
fn print_msg(){
  println!("invalid cli!");
  println!("required flags: -s <s> -E <E> -b <b> -t <tracefile>");
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
}
