pragma solidity ^0.6.0;

contract SystemContract {
    modifier systemContract(address addr) {
        bytes memory addrBytes = abi.encodePacked(addr);
        for (uint i = 0; i < 12; i++) {
            require(addrBytes[i] == 0, "not a system contract");
        }
        _;
    }

    modifier systemContracts(address[] memory addrs) {
        for (uint i = 0; i < addrs.length; i++) {
            bytes memory addrBytes = abi.encodePacked(addrs[i]);
            for (uint j = 0; j < 12; j++) {
                require(addrBytes[j] == 0, "not a system contract");
            }
        }
        _;
    }
}
