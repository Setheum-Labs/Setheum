# ECDP Emergency Shutdown Module

## Overview

Provides an Emergency Shutdown protocol for ECDP Stablecoin on Ethical DeFi. When a black swan occurs such as price plunge or fatal bug, the highest priority is to minimize user losses as much as possible. 

The Emergency Shutdown Module enables three types of Emergency Shutdown's which are:
- Setter Emergency Shutdown: To Shutdown the Setter (SETR) Stablecoin ECDP System.
- Slick USD Emergency Shutdown: To Shutdown the Slick USD (USSD) Stablecoin ECDP System.

When the decision to shutdown the system is made, emergency shutdown module needs to trigger all related modules to halt, and start a series of operations including close some user entry, freeze feed prices, run offchain worker to settle CDPs that have debit, cancel all activities in the auctions module, when debits and gaps are settled, the stable currency holders are allowed to refund the remaining collateral assets.
