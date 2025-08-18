# Transactions to Accounts Converter

## Next steps

I'd love to add some automated tests to make sure everything works as I expect it.

I'd love to take some more time to refactor the handlers in `main.rs` as they have
quite some duplication.

I'd love to add a profiler and consider concurrency/parallelism & the sorting approach
I outlined below to make this more performant.

## Approach

### For starters, I'll avoid optimizing for performance through concurrency/parallelism.

I've researched the 1B row challenge for this.
I've found they often do a number of optimizations like:
- splitting rows by bytes instead of using Strings
- using `crossbeam-channel` vs `std::sync::mpsc` as they are more feature rich and performant
- using `hashbrown` instead of the `std::collections::HashMap` (again, more performant)

If you read the approaches I've outlined, likely `crossbeam-channel` doesn't really
apply to them as much. More likely, you might use a Mutex or RwLock on a HashMap.
Or, maybe, you could use a `DashMap` from the `dashmap` crate (which allows us
to "shard" the data in buckets for concurrent use).

I won't do any of these optimizations for this take-home.
I think a higher initial priority is making sure we correctly scan through
the transactions to find and apply disputes/resolves/chargebacks correctly.

### How should we tackle the `transaction -> dispute/resolution/chargeback` mapping?

Note: for the sake of this discussion, I'll call disputes/resolves/chargebacks
"effects" not "transactions".

**TL;DR:** If you read below, you will understand the various approaches I've considered.
In my decision to take the first approach presented, I've assumed:
- You don't want me to prevent fraud
- You want me to apply transactions and **effects** chronologically
- The provided CSV will not be immense
- I'd rather use more memory than disk I/O for speed and simplicity
- Simpler is better for the sake of this take-home
I have though documented both how I could prevent fraud and reduce memory usage.

I've spent quite some time thinking about this part.
There is one key assumption that is, at this moment, messing with me.

Let's imagine this chronological situation:
1. user deposits $100 (tx1)
2. user withdraws $50 (tx2)
3. user disputes tx1

If processed chronologically, we allow potential fraud.
There is nowhere in the PDF that tells me to prevent this.
But I have found a few ways to prevent it and I like some of those designs.

#### If you don't want to prevent fraud...

In this case you have to process transactions for each client as they are received
from the initial CSV file, assumed chronologically.

**First approach**

One way is to loop through all rows, keeping a HashMap of `Client` structs and updating their balances as you go while also keeping a HashMap of `Transaction` structs to check if an
**effect** is valid (i.e. has a transaction existent previous to it, the tx has the right status to apply the new **effect** to it).

Imagine the `Transaction` struct contains the following fields:
`{ tx_id, client_id, amount, tx_type, dispute_status }`.

This is simple & straightforward, it allows for concurrency if you don't touch the
same client at once, but it comes with a big memory cost (need to keep all clients and transactions in memory).

**Second approach**

Another way, which is more memory efficient, but introduces disk I/O, is to
loop through all rows and sort them by `client_id` in chunks (big enough to fit in memory).
Once you sort a chunk, you write it to a temp file. Then you merge these files.

After merging, you can iterate through these sorted transactions to calculate client balances.
The magic here is that all of a client's transactions and **effects** will be adjacent,
so you don't have to hold them in memory for long. You flush them as soon as
you're done with the client.

In the first way, you had to keep all clients in memory because you didn't know for a fact whether new transactions/**effects** for existent clients would come up later.

#### If you want to prevent fraud...

If you want to prevent the fraud, it's important to only apply transactions to
balances once you've gone through all transactions and their **effects**.
This way you know the final state diff that a transaction will cause
(whether it is valid/disputed/resolved/chargedback).

**Third approach**

One way you can do that is to loop through all rows, keeping a HashMap of
`Transaction` structs. You update them with any **effects** you find but you do not
keep `Client` balances yet.

At the end, you iterate through these new "processed" transactions
to calculate client balances. This will prevent fraudulent withdrawals
because the **effects** to a transaction are applied before the withdrawal is processed.
I.e.: a chargebacked deposit won't apply.

This can also result in high memory usage if there are a lot of rows though
since you will keep all transactions in memory until the end of the process.

**Fourth approach**

Similarly as in the "don't prevent fraud" section, the other way here is
more memory efficient, but introduces disk I/O. You loop through all rows
but this time you sort them by `tx_id` in chunks (big enough to fit in memory).
Once you sort a chunk, you write it to a temp file. Then you merge these files.

After the merge, you can iterate through these sorted transactions to calculate client balances.
The magic here is that a transaction and all its **effects** will be adjacent, so you don't
have to hold them in memory for long if you apply them to the client's balance as you go.
You can flush them as soon as you are done with that `tx_id`.

In the third approach, you had to keep all transactions in memory because you
needed to make sure an **effect** doesn't come up later for a given `tx_id`.

## Background (no need to read this if you know the PDF already)

Takes in a CSV file similar to the ones in `data/tx`.
These will contain transactions in the format of:

| Type       | Client      | Tx     | Amount |
|------------|-------------|--------|--------|
| Deposit    | 1           | 1      | 100.00 |
| Withdrawal | 2           | 2      | -50.00 |

Client IDs are unique per client obviously.
Transaction IDs are unique globally (not just per client).
Transactions are not sorted by Client ID nor Transaction ID.
They are assumed to be sorted chronologically.
The CSV may contain whitespaces.
Decimal precision is limited to 4 decimal places.

The output will be a list of accounts in the format of:

| Client      | Available   | Held      | Total     | Locked    |
|-------------|-------------|-----------|-----------|-----------|
| 1           | 100.00      | 0.00      | 100.00    | False     |
| 2           | 50.00       | 0.00      | 50.00     | False     |

Available = funds available for trading, staking, or withdrawal.
Held = funds held for dispute.
Total = available + held
Locked = whether the account is locked due to a charge back.

A few invariants:
- `available = total - held`
- `held = total - available`
- `total = available + held`

The output shall look like this below, but note that the whitespaces and ordering are not important.

```
client,available,held,total,locked
2,2,0,2,false
1,1.5,0,1.5,false
```

### Types of transactions

There can be more than just `deposit` and `withdrawal` which are self-explanatory.
A `deposit` increases the `available` and `total` balances.
A `withdrawal` decreases the `available` and `total` balances.
A `dispute` increases the `held` balance and decreases the `available` balance.
A `resolve` decreases the `held` balance and increases the `available` balance.
A `chargeback` decreases the `total` balance and increases the `held` balance.

More interestingly, let's talk about `dispute`, `resolve`, and `chargeback`.

**A dispute** is a client's claim that a transaction was bad and should be reversed.
Our system shouldn't reverse it yet but the associated funds should be held.
This means that the client's:
- available funds should decrease by the amount disputed
- their held funds should increase by the amount disputed
- their total funds should remain the same

It's important to note that a dispute references an existent transaction.
It doesn't have an amount specified.
If the tx specified by the dispute doesn't exist you can ignore it.

An example of a dispute:

```
type, client, tx, amount
dispute, 1, 1,
```

**A resolve** is a resolution of a dispute, releasing the held funds.
This means that the client's:
- held funds should decrease by the amount no longer disputed
- their available funds should increase by the amount no longer disputed
- their total funds should remain the same

Like disputes, resolves also do not specify an amount but instead reference an existent transaction.

```
type, client, tx, amount
resolve, 1, 1,
```

Important to note and a potential issue if you just summed transfers up crudely:
If the tx specified doesn't exist, or the tx isn't under dispute, you can ignore the resolve.

**A chargeback** is the final step in a dispute and represents the client reversing a transaction.
Funds that were held have now been withdrawn.
This means that the client's:
- held funds and total funds should decrease by the amount previously disputed.

If a chargeback occurs the client's account should be immediately frozen.

Like a dispute and a resolve a chargeback refers to the transaction by ID (tx) and does not
specify an amount. Like a resolve, if the tx specified doesn't exist, or the tx isn't under dispute,
you can ignore chargeback.

```
type, client, tx, amount
chargeback, 1, 1,
```

### Other assumptions and notes

- this is all single-asset
- there are multiple clients, transactions reference clients, if a clien't doesn't exist we should "create a new record"
- client IDs are `u16` integers, no other metadata exists
- transaction IDs are `u32` integers
- it's recommended that we stream values instead of loading the whole dataset as it may be large
- code "cleanliness" is more important than performance in this exercise
- unlike withdrawals which should be prevented, disputes are processed even if they would make the available balance negative.

When in doubt on how to interpret a requirement, try to make assumptions that make sense for
a bank (think an ATM or more elaborate transaction processors), and document them.
