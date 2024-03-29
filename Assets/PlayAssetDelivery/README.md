Configuration for integrating Play Asset Delivery with unity addressables.

- Scene Loading in unity currently freezes forever on Android App Bundle builds if you have too many assets (>500MB?)
- Unity provides sample code for this, but it is incredibly buggy and broken.
- They are allegedly going to release a new version eventually that actually works
- Their code does not create the right directory structure.
- Unity changes this in a minor release and didn't fix their samples.
- The bundles are supposed to go under src/main/assets in each pack
- See https://forum.unity.com/threads/pad-unity-2021-3-not-created-src-main-assets-automatically-for-custom-asset-bundle.1389945/#post-9131950
- Attempted to fix their code to create the expected directory structure
- Even with this fix, PlayAssetDeliveryAssetBundleProvider is doing something wrong with its async operations
- Attempting to load assets gives an error
- "Exception: The ProvideHandle is invalid. After the handle has been completed, it can no longer be used."
- Google provides their own Unity plugin which works directly with AssetBundles
- It might be easier to just roll something on top of Google's solution, maybe they're more competent than Unity
- Fast Follow asset packs have a maximum size of 512MB, need to split them up
- Install Time asset pack has a max size of 1024MB, currently at 1262MB, could probably get under that limit
- .AAB bundle uploads always get rejected unless you are targeting the latest android API version
- You need to put the keystore password from 1password into Unity every time you launch in order to build the bundle
- You need to increment the 'android build version code' in player settings every time you do a build
- Doing development with asset bundles set to "on demand" *does* work
- It's just "fast follow" that is broken with the invalid handle error
- Releases through they play store still do seem to have issues with fetching bundles

- Honestly if Unity hasn't released a non-horrible asset management solution by the end of 2023, just write your own thing directly on top of the Apple and Google APIs and just use the addressables tools to build the asset bundles themselves. It seems like that is what most people are doing.
