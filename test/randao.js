const Randao = artifacts.require('Randao');

contract('Randao', (accounts) => {
  const founder = accounts[0];
  let randao, bnum, campaignID, commit, commitBalkline,
    commitDeadline, commitment, deposit, secret;

  beforeEach(async () => {
    randao = await Randao.new();
  });

  it('sets the founder on deployment', async () => {
    const deployedFounder = await randao.founder.call();
    assert.equal(founder, deployedFounder);
  });

  describe('newCampaign', () => {
    context('with valid timeline and deposit', () => {
      beforeEach(async () => {
        bnum = await web3.eth.getBlock("latest");
        bnum = bnum.number + 20;
        commitBalkline = 12;
        commitDeadline = 6;
        deposit = web3.utils.toWei('10', 'ether');
      });

      it('adds a new campaign', async () => {
        await randao.newCampaign(bnum, deposit, commitBalkline, commitDeadline, {from: founder, value: deposit});
        const campaigns = await randao.numCampaigns.call();
        assert.equal(campaigns.toString(), "1");
      });
    });
  });
});
