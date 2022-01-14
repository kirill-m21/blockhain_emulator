mod blockchain;
use blockchain::Blockchain;
use rand::Rng;

fn main() {
    let mut blnch: Blockchain = Blockchain::new(); //override ctor?

    for _index in 0..3 {
        blnch.new_transaction(
            "Sender".to_string(),
            "Receiver".to_string(),
            rand::thread_rng().gen_range(0..100_000_000 as u64),
        );
        blnch.mint();
    }

    println!("{:#?}", blnch);
}
