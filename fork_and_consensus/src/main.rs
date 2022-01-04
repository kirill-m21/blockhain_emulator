mod blockchain;
use blockchain::Blockchain;

fn main() {
    let mut blnch: Blockchain = Blockchain::new(); //override ctor?

    blnch.fork(300); // will stop if the largest chain is found or time is out

    println!("\nBlockchain len: {}", blnch.chain.len());
}
