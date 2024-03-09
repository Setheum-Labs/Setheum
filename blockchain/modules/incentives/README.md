# Incentives Module

Provides Incentive mechanisms for Ethical DeFi Protocols.

## Overview

Edfis Exchange needs to support multiple open liquidity reward mechanisms. Each Pool has  its own multi currencies rewards and reward accumulation mechanism. ORML rewards module records the total shares, total multi currencies rewards and user shares of specific pool.  Incentives module provides hooks to other protocals to manage shares, accumulates rewards and distributes rewards to users based on their shares.

## Pool types:

1. EcdpSetrLiquidityRewards: record the shares and rewards for Setter (SETR) ECDP users who are staking LP tokens.
2. EcdpUssdLiquidityRewards: record the shares and rewards for Slick USD (USSD) ECDP users who are staking LP tokens.
3. EdfisLiquidityRewards: record the shares and rewards for Edfis makers who are staking LP token.
4. EdfisXLiquidityRewards: record the shares and rewards for Edfis X (Cross-chain) makers who are staking XLP token.
5. MoyaEarnRewards: record the shares and rewards for users of Moya Earn (Moya Liquid Staking Protocol).

## Rewards accumulation:

Rewards: periodicly(AccumulatePeriod), accumulate fixed amount according to Rewards. Rewards come from RewardsSource, please transfer enough tokens to RewardsSource before start Rewards plan.
