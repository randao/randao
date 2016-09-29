'use script'

const Web3 = require('web3')
const Randao = require(`../build/contracts/Randao.sol.js`)
const CampaignCreatorDaemon = require('./campaign_creator_daemon')

const web3Provider = new Web3.providers.HttpProvider(`http://localhost:4500`)
const web3 = new Web3(web3Provider)
console.log(`web3 connected: ${web3.isConnected()}\n`)
web3.eth.defaultAccount = web3.eth.accounts[0]

Randao.setProvider(web3.currentProvider)
const randao = Randao.deployed()

const creator = new CampaignCreatorDaemon(web3, randao)
creator.start()
