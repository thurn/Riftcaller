// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#nullable enable

using System.Collections;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.ResourceManagement.AsyncOperations;

namespace Riftcaller.Assets
{
  public static class AssetUtil
  {
    public static void PlayOneShot(AudioSource source, AudioClip? clip)
    {
      if (clip)
      {
        source.PlayOneShot(clip!);
      }
    }
    
    public static IEnumerator PlayReferenceOneShot(AudioSource source, AssetReference reference)
    {
      var operation = Addressables.LoadAssetAsync<AudioClip>(reference);
      yield return operation;
      if (operation.Status == AsyncOperationStatus.Succeeded)
      {
        PlayOneShot(source, operation.Result);
      }
      else
      {
        LogUtils.LogError($"Error loading asset {reference}");
      }
    }
  }
}