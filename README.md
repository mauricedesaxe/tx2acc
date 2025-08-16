# Transactions to Accounts Converter

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

When in doubt on how to interpret a requirement, try to make assumptions that make sense for
a bank (think an ATM or more elaborate transaction processors), and document them.
