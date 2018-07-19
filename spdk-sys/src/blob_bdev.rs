use generated::spdk_blob_bdev_bindings::{
    spdk_bdev_create_bs_dev  
};
// struct BDev {

// }

// pub fn init(bdev_name: S) -> impl Future<()>
// where S: Into<String> {
//     unimplemented!("Oops");
// }

// pub fn bdev_get_by_name(bdev_name : S) 
// where S: Into<String> {
//     struct hello_context_t *hello_context = arg1;
// 	struct spdk_bdev *bdev = NULL;
// 	struct spdk_bs_dev *bs_dev = NULL;

// 	SPDK_NOTICELOG("entry\n");
// 	/*
// 	 * Get the bdev. For this example it is our malloc (RAM)
// 	 * disk configured via hello_blob.conf that was passed
// 	 * in when we started the SPDK app framework so we can
// 	 * get it via its name.
// 	 */
// 	bdev = spdk_bdev_get_by_name("Malloc0");
// 	if (bdev == NULL) {
// 	}

// 	bs_dev = spdk_bdev_create_bs_dev(bdev, NULL, NULL);
// 	if (bs_dev == NULL) {
// 	}

// 	//spdk_bs_init(bs_dev, NULL, bs_init_complete, hello_context);

//     // spdk_bdev *bdev
//     // spdk_bs_dev *bs_dev
//     let bdev_name_cstring = CString::new(bdev_name)
//         .expect("Couldn't create a string");
//     unsafe { 
//         spdk_bdev_get_by_name(bdev_name_cstring) 
//     }
// }