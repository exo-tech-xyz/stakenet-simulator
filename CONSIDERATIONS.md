## Important Points

The steward program had some upgradations recently adding few fields in the `ValidatorHistoryEntry` struct and the current logic of the program 
isn't backward compatible with the epochs before the update.

These values have been modified from the original data of the Validator History Program while data ingestion - 

1) `priority_fee_merkle_root_upload_authority` has been set to Unset (`u8::MAX`) for calculation purposes.

2) For epochs before 735, the all values of `priority_fee_commission` is `u16:MAX` and `priority_fee_tips` of `u64::MAX`. Most probably because these fields were not tracked by the program before this epoch.
These columns in the rows are manually set to 0 while data ingestion.

## Issues
Apart from the above mentioned keys, `total_leader_slots` was also not tracked before epoch 735, hence all the values in the program are `u32::MAX`. This makes the `delinquency_score` -> 0 as it basically depends on the ratio of `epoch_credits` and `total_leader_slots`.

This still hasn't been fixed in data ingestion as we still haven't figured out an ideal way to correct the `epoch_credits`/`total_leader_slots` ratio.
But this is known for the fact that correcting this ratio fixes the `delinquency_score` issue. (I personally set `epoch_credits` to `6870613` and `total_leader_slots` as `252` to solve it)

## Benchmarks

Beacuse of the above mentioned issue, running benchmarks would ideally fail if there is no manual change of data to fix the `delinquency_score` to not be 0.

Assuming there is no change in the data, any benchmarks, that tests after epoch 735 (740 to be safe :) as we take historyical epochs also to calculate scores.) should run fine.

If there are manual changes to the ratio for the epochs before 735, it has been observed that it would change the accuracy of the backtesting by a lot for that test, which doesn't make sense to do. We are still looking for an ideal way if we could solve this which the current limitations on the data.

> Note: We consider random validators to provide/withdraw stake on every run of the backtesting, hence we might have a very slight change in the APY calculations every time backtest is run (usually observed in order of `10^-3` which could be vary small but definitely a point to consider).


## Accuracy (after epoch 735)
The backtest always calculation of the APY was within `0.2% - 0.4%` range from the value calculated by the API. Interestingly, it was always always on the lesser side within `0.3 %` which we are trying to figure out the reason of.

Results of a run - 

| Epoch Range | Jito APY (%) | Backtest APY (%) |
|--------------|--------------|------------------|
| 735–800      | 8.3957       | 7.9782           |
| 732–800      | 8.4801       | 8.1432           |
| 750–850      | 7.5288       | 7.3033           |
| 800–850      | 7.0709       | 6.6263           |
| 740–850      | 7.6829       | 7.3851           |
