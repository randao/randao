# Findora Randao Implement
Randao implementation for Findora BlockChain.

## Findora Rosetta Docker build and run
### Findora Rosetta contract build and deploy
```bash
cd eth
REPORT_GAS=true
npm install --save-dev hardhat
npm install
npx hardhat compile
npx hardhat run scripts/deploy.ts --network localhost
```
### Findora Rosetta image build and run single container
```bash
docker build . -t findora-randao
docker run -p 80:80 -p 8080:8080 -p 9090:9090 \
-v $(pwd)/campaigns/participant0:/tmp/.randao/campaigns \
-v $(pwd)/config/config0.json:/tmp/.randao/config/prinet/config0.json \
-v $(pwd)/keys:/tmp/.randao/keys \
-itd --name findora-randao --restart always findora-randao
```
### Findora Rosetta image build and multi run multiple container
```bash
docker-compose up -d
```

