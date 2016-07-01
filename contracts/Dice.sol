// Sample code
contract Dice {
  uint256 public random;
  address public randao;

  function Dice(address _randao) {
      randao = _randao;
  }

  /*
    newCampaign(1808247, 20 ether, 20, 10)
  */
  function newCampaign(
      uint32 _targetBlockNum,
      uint96 _deposit,
      uint8 _commitBalkline,
      uint8 _commitDeadline) {
      randao.call.value(20 ether)(bytes4(sha3("newCampaign(uint32,uint96,uint8,uint8)")), _targetBlockNum, _deposit, _commitBalkline, _commitDeadline);
  }
  /*
    call returns a boolean indicating whether the invoked function terminated (true) or caused an EVM exception (false).
    It is not possible to access the actual data returned (for this we would need to know the encoding and size in advance).
    So, pleace check up the campaignID on etherscan.io or Mist.
  */
}
