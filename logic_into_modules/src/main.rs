mod blockchain;
use blockchain::Blockchain;

fn main() {
    let mut blnch: Blockchain = Blockchain::new(); //override ctor?

    blnch.new_transaction(3); //queues transactions with a random amount

    blnch.mint();
    blnch.mint();

    println!("{:#?}", blnch.chain);
    println!("--------------------");
    println!("{:#?}", blnch.tr_queue);
}
