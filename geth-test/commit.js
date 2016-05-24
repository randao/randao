var zerostr = Array.apply(null, Array(3)).map(String.prototype.valueOf, "0").join('');
var s = 'a';
var secret = '0x' + (zerostr + web3.toHex(s).substr(2)).substr(-64, 64);

var commitment = web3.sha3(secret, true);
console.log('commitment: ', commitment);

randao.commit.sendTransaction(campaignID - 1, commitment, {value: deposit, from: eth.accounts[2]});

console.log('commit plz wait...');
console.log('commit at blockNumber: ', web3.eth.blockNumber);

miner.start(); admin.sleepBlocks(1); miner.stop();
console.log('campaigns: ', randao.campaigns.call());
// TODO: wrong commitment
console.log('get commitment: ', randao.getCommitment.call(campaignID - 1));
