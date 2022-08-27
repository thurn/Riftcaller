// Copyright © Spelldawn 2021-present

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
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.AddressableAssets;

namespace Spelldawn.Assets
{
  public sealed class LazySprite: MonoBehaviour
  {
    [SerializeField] AssetReferenceSprite _sprite = null!;

    IEnumerator Start()
    {
      if (AssetPreference.UseProductionAssets)
      {
        var operation = Addressables.LoadAssetAsync<Sprite>(_sprite);
        yield return operation;
        ComponentUtils.GetComponent<SpriteRenderer>(gameObject).sprite = operation.Result;        
      }
    }
  }
}