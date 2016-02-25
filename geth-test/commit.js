
var zerostr = Array.apply(null, Array(3)).map(String.prototype.valueOf,"0").join('');
var s = 'a';
var secret = '0x' + (zerostr + web3.toHex(s).substr(2)).substr(-64, 64);
var deposit = web3.toWei('2', 'ether');
var commitment = '0x' + web3.sha3(s);

randao.commit.sendTransaction(campaignID, commitment, {value: web3.toWei('10', 'ether'), from: eth.accounts[2]});

console.log('commit plz wait...');

miner.start(); admin.sleepBlocks(1); miner.stop();
