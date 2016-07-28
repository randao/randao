contract Sha3 {
    function commit(uint256 _s) returns (bytes32) {
        return sha3(_s);
    }
}
