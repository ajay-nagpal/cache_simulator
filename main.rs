use std::env;                 //read command line arguments
use getopts::Options;              //Cli arguments parsing

/*
Function to print invalid format message when entered  cli is invalid
*/
fn print_msg(){
  println!("invalid cli!");
  println!("required flags in order: -s <s> -E <E> -b <b> -t <tracefile>");
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
}
