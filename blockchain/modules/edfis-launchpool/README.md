# Edfis Launchpool Module
Edfis Launchpool is a platform for projects to offer their tokens to their community (IDO) on Edfis while listing their liquidity pool on the exchange.

## Overview

This module is used for Edfis Launchpools. Teams and projects that are just getting started launching their products would need to foster a community and provide their tokens to the public. They need community backed by token holders of their token, that is the crowd so that they could have a strong start. 

The Launchpool does not replace the Launchpad but is however complementary to it. The Launchpool module allows teams to offer their tokens to users in a pool wherein users stake any currency in order to earn the team's new token passively.

Based on the `LaunchpoolStakingCurrency` set by teams, the protocol takes a `LaunchpoolCommission` from the tokens offered. The commissions taken are deposited to the `EthicalDeFiTreasury`.

### Launchpool Commission Teers
Below are the commission teers:
 - 1. `LSEE` Pools: No Commissions taken;
 - 2. `LEDF` Pools: No Commissions taken;
 - 3. `SETR` Pools: No Commissions taken;
 - 4. `USSD` Pools: `USSDLaunchpoolCommission` (should be 0.25%)
 - 5. `SEE` Pools: `SEELaunchpoolCommission` (should be 0.5%);
 - 6. `EDF` Pools: `EDFLaunchpoolCommission` (should be 0.5%)
 - 7. Other Tokens Pools: `OtherTokensLaunchpoolCommission` (should be 5%)
