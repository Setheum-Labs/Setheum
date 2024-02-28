# Transaction Payment Module

## Overview

Transaction payment module is responsible for charge fee and tip in different currencies. It provides a `MultiCurrency` payment that is settled into the `NativeCurrency` SEE by using internal sub account swapping pools or swapping on the Edfis DEX.
