بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

# Payments Module

This module allows users to create secure reversible payments that keep funds locked in a merchant's account until the off-chain goods are confirmed to be received. Each payment gets assigned its own *judge* that can help resolve any disputes between the two parties.

## Terminology

- Created: A payment has been created and the amount arrived to its destination but it's locked.
- NeedsReview: The payment has bee disputed and is awaiting settlement by a judge.
- IncentivePercentage: A small share of the payment amount is held in escrow until a payment is completed/cancelled. The Incentive Percentage represents this value.
- Resolver Account: A resolver account is assigned to every payment created, this account has the privilege to cancel/release a payment that has been disputed.
- Remark: The module allows to create payments by optionally providing some extra(limited) amount of bytes, this is referred to as Remark. This can be used by a marketplace to separate/tag payments.
- CancelBufferBlockLength: This is the time window where the recipient can dispute a cancellation request from the payment creator.


## Transfers

Create these measures as a part of `Edfis Pay` so that users can opt-in and out at will.
1. `ensure_account_exists()`: If the account doesn't exist, fail the transaction.
2. `ensure-account_has_ed()`: If the account doesn't have ED, fail the transaction.

Provide various types of transfer options to users, each of which must handle both `ensure_account_exists()` and `account_has_ed()` options (they are optional for users to use, so they should be of type `Boolean`).

### Transfer Types that Handle this issue:

1. `red_packet_transfer`: to `automatically_unlock` the funds to the sender if the receiver does not opened the red packet within `TransferUnlockPeriod` , therefore reversing the transaction.
2. `reclaimable_transfer`: allows the sender to `reclaim` (unlock) the funds if the receiver does not `claim` the transfer within `TransferUnlockPeriod` , therefore reversing the transaction.
3. `willing_transfer`: allows the receiver to `accept` or `reject` the funds.
4. `reversible_willing_transfer`: allows the receiver to `accept` or `reject` the funds. The transfer will automatically unlock if the receiver does not `claim` the transfer within `TransferUnlockPeriod` , therefore reversing the transaction.
5. `protected_transfer`: allows the receiver to `claim` the transfer only if the receiver knows the `password` to the transfer, else the transfer cannot be claimed therefore the `TransferStatus` stays as `Unclaimed`.
6. `reversible_protected_transfer`: allows the receiver to `claim` the transfer only if the receiver knows the `password` to the transfer, else the transfer cannot be claimed. The transfer will `automatically_unlock` the funds to the sender if the receiver does not claim the transfer within `TransferUnlockPeriod`.

## Interface

#### Events

- `PaymentCreated { from: T::AccountId, asset: AssetIdOf<T>, amount: BalanceOf<T> },`,
- `PaymentReleased { from: T::AccountId, to: T::AccountId }`,
- `PaymentCancelled { from: T::AccountId, to: T::AccountId }`,
- `PaymentCreatorRequestedRefund { from: T::AccountId, to: T::AccountId, expiry: BlockNumberFor<T>}`
- `PaymentRefundDisputed { from: T::AccountId, to: T::AccountId }`
- `PaymentRequestCreated { from: T::AccountId, to: T::AccountId }`
- `PaymentRequestCompleted { from: T::AccountId, to: T::AccountId }`

#### Extrinsics

- `pay` - Create an payment for the given currencyid/amount
- `pay_with_remark` - Create a payment with a remark, can be used to tag payments
- `release` - Release the payment amount to recipent
- `cancel` - Allows the recipient to cancel the payment and release the payment amount to creator
- `resolve_release_payment` - Allows assigned judge to release a payment
- `resolve_cancel_payment` - Allows assigned judge to cancel a payment
- `request_refund` - Allows the creator of the payment to trigger cancel with a buffer time.
- `claim_refund` - Allows the creator to claim payment refund after buffer time
- `dispute_refund` - Allows the recipient to dispute the payment request of sender
- `request_payment` - Create a payment that can be completed by the sender using the `accept_and_pay` extrinsic.
- `accept_and_pay` - Allows the sender to fulfill a payment request created by a recipient

## Implementations

The RatesProvider module provides implementations for the following traits.
- [`PaymentHandler`](./src/types.rs)

## Types

The `PaymentDetail` struct stores information about the payment/escrow. A "payment" in Setheum Pay is similar to an escrow, it is used to guarantee proof of funds and can be released once an agreed upon condition has reached between the payment creator and recipient. The payment lifecycle is tracked using the state field.

```rust
pub struct PaymentDetail<T: pallet::Config> {
	/// type of asset used for payment
	pub asset: AssetIdOf<T>,
	/// amount of asset used for payment
	pub amount: BalanceOf<T>,
	/// incentive amount that is credited to creator for resolving
	pub incentive_amount: BalanceOf<T>,
	/// enum to track payment lifecycle [Created, NeedsReview]
	pub state: PaymentState<BlockNumberFor<T>>,
	/// account that can settle any disputes created in the payment
	pub resolver_account: T::AccountId,
	/// fee charged and recipient account details
	pub fee_detail: Option<(T::AccountId, BalanceOf<T>)>,
	/// remarks to give context to payment
	pub remark: Option<BoundedDataOf<T>>,
}
```

The `PaymentState` enum tracks the possible states that a payment can be in. When a payment is 'completed' or 'cancelled' it is removed from storage and hence not tracked by a state.

```rust
pub enum PaymentState<BlockNumber> {
	/// Amounts have been reserved and waiting for release/cancel
	Created,
	/// A judge needs to review and release manually
	NeedsReview,
	/// The user has requested refund and will be processed by `BlockNumber`
	RefundRequested(BlockNumber),
}
```

## GenesisConfig

The rates_provider pallet does not depend on the `GenesisConfig`
