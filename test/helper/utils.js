var utils = {
  prepare4reveals: function(randao, accounts, campaignID) {
    var zerostr = new Array(64).fill('0').join('');
    var secrets = ['5', '9', '111', '12'].map((s) => { return '0x' + (zerostr + web3.toHex(s).substr(2)).substr(-64, 64); });
    var deposit = web3.toWei('2', 'ether');
    var commitments = secrets.map((s) => { web3.sha3(s, true); });

    var promise  = Promise.all(
      commitments.map((commitment, i) => {
        return randao.commit.sendTransaction(campaignID, commitment, {value: web3.toWei('10', 'ether'), from: accounts[0]});
    }));

    return [secrets, promise];
  }
}

module.exports = utils;
