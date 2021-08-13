use crate::mock::*;
use frame_support::{assert_err, assert_ok};
use sp_io::hashing::blake2_256;
use utilities::ClassId;

#[test]
fn create_class_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(NFT721Module::create_class(
            Origin::signed(1),
            vec![1],
            vec![2],
            1000
        ));
    })
}

#[test]
fn mint_nft_should_work() {
    new_test_ext().execute_with(|| {
        let did = blake2_256(b"test");
        let new_class_id = ClassId { did };
        assert_ok!(NFT721Module::mint_nft(
            Origin::signed(1),
            new_class_id,
            vec![1],
            vec![2],
            100
        ));
    })
}

#[test]
fn transfer_nft_should_work() {
    new_test_ext().execute_with(|| {
        let did = blake2_256(b"test");
        let new_class_id = ClassId { did };
        let nft_id = NFT721Module::_mint_nft(new_class_id, vec![1], vec![2], 100, 1);
        assert_eq(nft_id, None);
        assert_ok!(NFT721Module::transfer_nft(
            Origin::signed(1),
            1,
            2,
            nft_id.unwrap()
        ));
    })
}
