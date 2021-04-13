pragma solidity ^0.6.0;

import "./ISettinDex.sol";
import "../utils/SystemContract.sol";
import "../token/IMultiCurrency.sol";

contract SettinDex is SystemContract, ISettinDex {
    /**
     * @dev Get liquidity pool of the currency_id_a and currency_id_b.
     * Returns (liquidity_a, liquidity_b)
     */
    function getLiquidityPool(address tokenA, address tokenB)
    public
    view
    override
    systemContract(tokenA)
    systemContract(tokenB)
    returns (uint256, uint256)
    {
        require(tokenA != address(0), "SettinDex: tokenA is zero address");
        require(tokenB != address(0), "SettinDex: tokenB is zero address");

        uint256 currencyIdA = IMultiCurrency(tokenA).currencyId();
        uint256 currencyIdB = IMultiCurrency(tokenB).currencyId();

        uint input_size = 3;
        uint256[] memory input = new uint256[](input_size);

        input[0] = 0;
        input[1] = currencyIdA;
        input[2] = currencyIdB;

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        uint256[2] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000405, input, input_size_32, output, 0x40)
            ) {
                revert(0, 0)
            }
        }
        return (output[0], output[1]);
    }

    /**
     * @dev Get swap target amount.
     * Returns (target_amount)
     */
    function getSwapTargetAmount(address[] memory path, uint256 supplyAmount)
    public
    view
    override
    systemContracts(path)
    returns (uint256) {
        require(path.length >= 2 && path.length <= 3, "SettinDex: token path over the limit");
        for (uint i = 0; i < path.length; i++) {
            require(path[i] != address(0), "SettinDex: token is zero address");
        }
        require(supplyAmount != 0, "SettinDex: supplyAmount is zero");

        uint input_size = 3 + path.length;
        uint256[] memory input = new uint256[](input_size);

        input[0] = 1;
        input[1] = path.length;
        for (uint i = 0; i < path.length; i++) {
            input[2 + i] = IMultiCurrency(path[i]).currencyId();
        }
        input[input_size - 1] = supplyAmount;

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000405, input, input_size_32, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return output[0];
    }

    /**
     * @dev Get swap supply amount.
     * Returns (supply_amount)
     */
    function getSwapSupplyAmount(address[] memory path, uint256 targetAmount)
    public
    view
    override
    systemContracts(path)
    returns (uint256) {
        require(path.length >= 2 && path.length <= 3, "SettinDex: token path over the limit");
        for (uint i = 0; i < path.length; i++) {
            require(path[i] != address(0), "SettinDex: token is zero address");
        }
        require(targetAmount != 0, "SettinDex: targetAmount is zero");

        uint input_size = 3 + path.length;
        uint256[] memory input = new uint256[](input_size);

        input[0] = 2;
        input[1] = path.length;
        for (uint i = 0; i < path.length; i++) {
            input[2 + i] = IMultiCurrency(path[i]).currencyId();
        }
        input[input_size - 1] = targetAmount;

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000405, input, input_size_32, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        return output[0];
    }

    /**
     * @dev Swap with exact supply.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function swapWithExactSupply(address[] memory path, uint256 supplyAmount, uint256 minTargetAmount)
    public
    override
    systemContracts(path)
    returns (bool) {
        require(path.length >= 2 && path.length <= 3, "SettinDex: token path over the limit");
        for (uint i = 0; i < path.length; i++) {
            require(path[i] != address(0), "SettinDex: token is zero address");
        }
        require(supplyAmount != 0, "SettinDex: supplyAmount is zero");

        uint input_size = 5 + path.length;
        uint256[] memory input = new uint256[](input_size);

        input[0] = 3;
        input[1] = uint256(msg.sender);
        input[2] = path.length;
        for (uint i = 0; i < path.length; i++) {
            input[3 + i] = IMultiCurrency(path[i]).currencyId();
        }
        input[input_size - 2] = supplyAmount;
        input[input_size - 1] = minTargetAmount;

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000405, input, input_size_32, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        emit Swaped(msg.sender, path, supplyAmount, output[0]);
        return true;
    }

    /**
     * @dev Swap with exact target.
     * Returns a boolean value indicating whether the operation succeeded.
     */
    function swapWithExactTarget(address[] memory path, uint256 targetAmount, uint256 maxSupplyAmount)
    public
    override
    systemContracts(path)
    returns (bool) {
        require(path.length >= 2 && path.length <= 3, "SettinDex: token path over the limit");
        for (uint i = 0; i < path.length; i++) {
            require(path[i] != address(0), "SettinDex: token is zero address");
        }
        require(targetAmount != 0, "SettinDex: targetAmount is zero");

        uint input_size = 5 + path.length;
        uint256[] memory input = new uint256[](input_size);

        input[0] = 4;
        input[1] = uint256(msg.sender);
        input[2] = path.length;
        for (uint i = 0; i < path.length; i++) {
            input[3 + i] = IMultiCurrency(path[i]).currencyId();
        }
        input[input_size - 2] = targetAmount;
        input[input_size - 1] = maxSupplyAmount;

        // Dynamic arrays will add the array size to the front of the array, so need extra 1 size.
        uint input_size_32 = (input_size + 1) * 32;

        uint256[1] memory output;

        assembly {
            if iszero(
                staticcall(gas(), 0x0000000000000000405, input, input_size_32, output, 0x20)
            ) {
                revert(0, 0)
            }
        }
        emit Swaped(msg.sender, path, output[0], targetAmount);
        return true;
    }
}
