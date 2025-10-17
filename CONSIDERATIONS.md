## Important Points

The steward program had some upgrades recently, adding a few fields in the `ValidatorHistoryEntry` struct, and the current logic of the program isn't backward compatible with the epochs before the update.

These values have been modified from the original data of the Validator History Program during data ingestion —

1. `priority_fee_merkle_root_upload_authority` has been set to Unset (`u8::MAX`) for calculation purposes.

2. For epochs before 735, all values of `priority_fee_commission` are `u16::MAX` and `priority_fee_tips` are `u64::MAX`, most probably because these fields were not tracked by the program before this epoch.
   These columns in the rows are manually set to 0 during data ingestion.

## Issues

Apart from the above-mentioned issues, the `delinquency_score` is 0 for epochs below 735. This is causing the score of all the validators to be 0. Looking to the code, it seems it depends on `epoch_credits` and `blocks_produced`. I was able to resolve the issue by updating `epoch_credits` to a set number (6870613 in my case) but I'm still unsure of a valid fix for this as changing the epoch credits changes the APY by a lot which makes it pointless.

## Benchmarks

Because of the above-mentioned issue, running benchmarks would ideally fail if there is no manual change of data to fix the `delinquency_score` to not be 0.

Assuming there is no change in the data, any benchmarks that test after epoch 735 (740 to be safe :) as we take historical epochs also to calculate scores) should run fine.

If there are manual changes to the ratio for the epochs before 735, it has been observed that it would change the accuracy of the backtesting by a lot for that test, which doesn’t make sense to do. We are still looking for an ideal way to solve this within the current limitations on the data.

> Note: We consider random validators to provide/withdraw stake on every run of the backtesting; hence, we might have a very slight change in the APY calculations every time the backtest is run (usually observed in the order of `10^-3`, which could be very small but is definitely a point to consider).

## Accuracy (after epoch 735)

The backtest APY calculation was always within the `0.2% – 0.4%` range of the value calculated by the API. Interestingly, it was always on the lesser side within `0.3%`, which we are trying to figure out the reason for.

Results of a run —

| Epoch Range | Jito APY (%) | Backtest APY (%) |
| ----------- | ------------ | ---------------- |
| 735–800     | 8.3957       | 7.9782           |
| 732–800     | 8.4801       | 8.1432           |
| 750–850     | 7.5288       | 7.3033           |
| 800–850     | 7.0709       | 6.6263           |
| 740–850     | 7.6829       | 7.3851           |
