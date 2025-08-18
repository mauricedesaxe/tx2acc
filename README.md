# Transactions to Accounts Converter

Note: for the sake of this discussion, I'll call disputes/resolves/chargebacks
"effects" not "transactions".

## Table of Contents

- [Testing strategy](#testing-strategy)
- [How I've tackled the `transaction -> dispute/resolution/chargeback` mapping?](#how-ive-tackled-the-transaction---disputeresolutionchargeback-mapping)
  - [If you don't want to prevent fraud...](#if-you-dont-want-to-prevent-fraud)
  - [If you want to prevent fraud...](#if-you-want-to-prevent-fraud)
- [Various performance optimizations I could do](#various-performance-optimizations-i-could-do)
- [Other improvements I'd make](#other-improvements-id-make)
- [AI Usage](#ai-usage)
- [Background](#background-no-need-to-read-this-if-you-know-the-pdf-already)
  - [Types of transactions](#types-of-transactions)
  - [Other assumptions and notes](#other-assumptions-and-notes)

## Testing strategy

I focused on integration tests over unit tests. Rather than testing
individual functions, I test the main `handle_transaction` function with
(hopefully) realistic transaction sequences. This validates the complete flow
from raw transactions through balance updates and ensures all edge cases
work together correctly.

Currently I've covered simple deposits and withdrawals, a more complex
sequence with a mix of **effects** and whether we actually lock accounts.

This is by no means all it could be. One could add edge case tests for negative/zero
amount deposits, disputing a withdrawal, effects on non existent transactions,
effects before the transaction exists, duplicate effects, integer overflow
(someone depositing a huge amount which we then convert by `* 10000`).

There are truly quite some edge cases I didn't have time to cover.
For some of them, like "disputing a withdrawal", I haven't even
clearly defined what behaviour I want to see to myself.

I might also want to fuzz this and ensure some invariants are always true.

**Why I chose integration testing?** The logic in my case is tightly
coupled, I haven't coded very "functional" code, so this was easier
than trying to test each function in isolation. It also is all encompassing
and very similar to how you will test the system yourself.

## How I've tackled the `transaction -> dispute/resolution/chargeback` mapping?

In **my decision to take the first approach presented**, I've assumed:
- You don't want me to prevent fraud of the `deposit -> withdraw -> chargeback the deposit` type
- You want me to apply transactions and **effects** chronologically
- The provided CSV will not be immense
- You'd rather use more memory than disk I/O
- Simpler is better

I have though documented both how I could prevent fraud and reduce memory usage.
Also, there are other (smaller?) assumptions that you can read below.

I've spent quite some time thinking about this part.
There is one key assumption that is, at this moment, messing with me.

Let's imagine this chronological situation:
1. user deposits $100 (tx1)
2. user withdraws $50 (tx2)
3. user disputes tx1

If processed chronologically, we allow potential fraud.
There is nowhere in the PDF that tells me to prevent this.
But I have found a few ways to prevent it and I like some of those designs.

### If you don't want to prevent fraud...

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

### If you want to prevent fraud...

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

## Various performance optimizations I could do

I've researched the 1B row challenge for this.
I've found they often do a number of optimizations like:
- splitting rows by bytes instead of using Strings
- using `crossbeam-channel` vs `std::sync::mpsc` as they are more feature rich and performant
- using `hashbrown` instead of the `std::collections::HashMap` (again, more performant)

I'd personally not consider splitting rows by bytes.
Too complex & error-prone for my liking.

If you read the approaches I've outlined below, likely `crossbeam-channel` and channels
in general don't apply as much.

If I wanted to introduce concurrency, I might use a Mutex or RwLock on a HashMap.
But that's problematic. I'm not sure how much performance gain I'd get since
all operations touch those maps and you'd run into a lot of locking time.

More likely, I could use a `DashMap` from the `dashmap` crate which allows us
to "shard" the data in buckets for concurrent use. So one thread touches
clients 1-100, another touches clients 101-200, etc and they don't step
on each other's toes.

Of course, maybe there's another way to design this that doesn't have HashMaps?
So then maybe Mutex/RwLocks/channels make more sense.

More importantly, I'd love to add a profiler before I implement either
`DashMap` or the sorting approach outlined above. You can't improve
what you don't measure.

I haven't done any of these because of lack of time.

## Other improvements I'd make

I'd love to take some more time to refactor the handlers in `main.rs` as they have
quite some duplication.

I'd also take some time to fix those clippy errors and make CI pass.

## AI Usage

I have used AI (Claude, Zed, Perplexity, Copilot) throughout
this assignment for various things like:
- rubber ducking
- boilerplate implementation (like for CSV streaming)
- generation of samples & tests

I still wrote most of the code, have obviously closely reviewed everything that
was generated and fully own all of the code that got committed.

## Background

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
- duplicate transactions are to be ignored
- clients shouldn't be able to dispute other clients' transactions

When in doubt on how to interpret a requirement, try to make assumptions that make sense for
a bank (think an ATM or more elaborate transaction processors), and document them.
