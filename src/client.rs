#[derive(Debug, Clone)]
pub struct Client {
    client_id: u16,
    pub available: i64,
    pub held: i64,
    pub total: i64,
    pub locked: bool,
}

impl Client {
    pub fn new(client_id: u16) -> Self {
        Client {
            client_id,
            available: 0,
            held: 0,
            total: 0,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: i64) {
        self.available += amount;
        self.total += amount;
        eprintln!(
            "Client {} deposited {} in tx and now has these balances: available={}, held={}, total={}",
            self.client_id, amount, self.available, self.held, self.total
        );
    }

    pub fn withdraw(&mut self, amount: i64) {
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
            eprintln!(
                "Client {} withdrew {} and now has these balances: available={}, held={}, total={}",
                self.client_id, amount, self.available, self.held, self.total
            );
        } else {
            // Not yet sure yet how I should deal with this aside from
            // not changing the balance.
            eprintln!("User is trying to withdraw more than they have.");
        }
    }

    pub fn apply_dispute(&mut self, amount: i64) {
        self.available -= amount;
        self.held += amount;
        eprintln!(
            "Client {} applied dispute for {} and now has these balances: available={}, held={}, total={}",
            self.client_id, amount, self.available, self.held, self.total
        );
    }

    pub fn apply_resolve(&mut self, amount: i64) {
        self.available += amount;
        self.held -= amount;
        eprintln!(
            "Client {} resolved dispute for {} and now has these balances: available={}, held={}, total={}",
            self.client_id, amount, self.available, self.held, self.total
        );
    }

    pub fn apply_chargeback(&mut self, amount: i64) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
        eprintln!(
            "Client {} had chargeback for {} and now has these balances: available={}, held={}, total={}, locked={}",
            self.client_id, amount, self.available, self.held, self.total, self.locked
        );
    }
}
