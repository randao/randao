var secret = web3.toHex('abc').slice(2);
var commitment = '0x' + web3.sha3(secret, { encoding: 'hex' });

console.log('commitment: ', commitment);
miner.start(); admin.sleepBlocks(2); miner.stop();
randao.commit(campaignID - 1, commitment, {value: deposit, from: eth.accounts[2]});

console.log('commit plz wait...');
console.log('commit at blockNumber: ', web3.eth.blockNumber);

miner.start(); admin.sleepBlocks(1); miner.stop();
console.log('campaigns: ', randao.campaigns.call());
// TODO: wrong commitment
console.log('get commitment: ', randao.getCommitment.call(campaignID - 1, {from: eth.accounts[2]}));
