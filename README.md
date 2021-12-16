# NFT Marketplace

### Main features:
- Allow user list their NFT with a small bond
- Unlist NFT and get refund
- Get User NFTs
- Get All NFTs

### Demo Flow:
1. Deploy & init NFT contract and Marketplace contract
2. Mint 1 NFT token
3. NFT Owner call add_sale fn 
4. Approve marketplace to transfer your token
5. Someone go to purchase the NFT
6. Check buyer wallet

### 1 Deploy & Init
```sh
#Deploy NFT contract
cd NFT 
./build.sh
near dev-deploy --wasmFile res/non_fungible_token.wasm
source neardev/dev-account.env
echo $CONTRACT_NAME
near call $CONTRACT_NAME new_default_meta '{"owner_id": "'$CONTRACT_NAME'"}' --accountId $CONTRACT_NAME

#Deploy marketplace contract
cd ..
cd nft_marketplace
cargo build --all --target wasm32-unknown-unknown --release
cptarget/wasm32-unknown-unknown/release/*.wasm ./res/
near dev-deploy --wasmFile res/marketplace.wasm 
source neardev/dev-account.env
echo $ID
```

### 2 Mint NFT token
```sh
near call $CONTRACT_NAME nft_mint '{"token_id": "0", "receiver_id": "'$CONTRACT_NAME'", "token_metadata": { "title": "Olympus Mons", "description": "Tallest mountain in charted solar system", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 1}}' --accountId $CONTRACT_NAME --deposit 0.1
```

### 3 NFT owner call add_sale fn
```sh
near call $ID add_sale '{"token_contract_id": "'$CONTRACT_NAME'", "token_id": "0", "price": "1_000_000_000_000_000_000_000_000", "on_behalf_of": "'$CONTRACT_NAME'"}' --accountId $CONTRACT_NAME --deposit 0.1
```

### 4 Approve marketplace to transfer your token
```sh
near call $CONTRACT_NAME nft_approve \
  '{ "token_id": "0", "account_id": "'$ID'" }' \
  --accountId $CONTRACT_NAME --depositYocto 1
```

### 5 Purchase NFT
```sh
near call $ID purchase '{"token_contract_id": "'$CONTRACT_NAME'", "token_id": "0"}' --accountId nft_buyer.testnet
```

### 6 Buyer check his wallet
 
