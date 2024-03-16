# VK Storage Module

## Overview

This module provides a way to store verification keys that can be used in the SNARK verification process. Anybody can register a verification key. A key is stored in a map under its Blake256 hash. Pallet doesn't provide any way for removing keys from the map, so it's a good idea to impose some costs on storing a key (see `StorageCharge`) to avoid bloating the storage.
