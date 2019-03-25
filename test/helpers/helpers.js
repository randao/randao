const sendToEvm = async (evmMethod, ...params) => {
  await web3.currentProvider.send(
    {
      id: 0,
      jsonrpc: '2.0',
      method: evmMethod,
      params: [...params]
    },
    (error, result) => {
      if (error) {
        console.log(`Error during ${evmMethod}! ${error}`);
        throw error;
      }
    }
  );
};

const mineBlocks = async (blocks) => {
  for (let i = 0; i < blocks; i++) {
    await sendToEvm('evm_mine');
  }
};

const assertThrowsAsync = async (fn, regExp) => {
  let f = () => {};
  try {
    await fn();
  } catch(e) {
    f = () => {throw e};
  } finally {
    assert.throws(f, regExp);
  }
};

const setupNewCampaign = async (randao, consumer) => {
  let bnum = await web3.eth.getBlock("latest");
  bnum = bnum.number + 20;
  const commitBalkline = 12;
  const commitDeadline = 6;
  const deposit = web3.utils.toWei('10', 'ether');
  await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: consumer, value: deposit});
};

exports.assertThrowsAsync = assertThrowsAsync;
exports.mineBlocks = mineBlocks;
exports.setupNewCampaign = setupNewCampaign;
