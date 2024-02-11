// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract Erc20DemoContract is ERC20 {
    constructor(uint256 initialSupply) ERC20("long string name, long string name, long string name, long string name, long string name", "TestToken") {
        // mint alice 10000
        _mint(0x1000000000000000000000000000000000000001, 10000);
    }

    function decimals() public view virtual override returns (uint8) {
        return 17;
    }
}
