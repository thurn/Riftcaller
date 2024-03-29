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

using System;
using System.Collections;
using System.Collections.Generic;
using Riftcaller.Assets;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.ResourceManagement.AsyncOperations;

#nullable enable

namespace Riftcaller.Services
{
  public sealed class AssetPoolService : MonoBehaviour
  {
    readonly Dictionary<int, List<GameObject>> _pools = new();

    public T Create<T>(T prefab, Vector3 position, Transform? parent = null) where T : Component =>
      ComponentUtils.GetComponent<T>(Create(prefab.gameObject, position, parent));

    public IEnumerator CreateFromReference(
      AssetReferenceGameObject prefabReference,
      Vector3 position,
      Transform? parent = null,
      Action<GameObject>? onCreate = null)
    {
      if (UseProductionAssets.ShouldUseProductionAssets)
      {
        var operation = Addressables.LoadAssetAsync<GameObject>(prefabReference);
        yield return operation;
        if (operation.Status == AsyncOperationStatus.Succeeded)
        {
          var result = Create(operation.Result, position, parent);
          onCreate?.Invoke(result);
        }
        else
        {
          LogUtils.LogError($"Error: Failed to load asset {prefabReference}");
        }
      }
    }

    public GameObject Create(GameObject prefab, Vector3 position, Transform? parent = null)
    {
      var instanceId = prefab.GetInstanceID();
      if (_pools.ContainsKey(instanceId))
      {
        var list = _pools[instanceId];
        foreach (var pooledObject in list)
        {
          if (!pooledObject.activeSelf)
          {
            pooledObject.transform.SetParent(parent, worldPositionStays: false);
            pooledObject.transform.position = position;
            pooledObject.SetActive(value: true);
            return pooledObject;
          }
        }
      }
      else
      {
        _pools[instanceId] = new List<GameObject>();
      }

      var result = Instantiate(prefab, parent, worldPositionStays: false);
      result.transform.position = position;
      _pools[instanceId].Add(result.gameObject);
      return result;
    }
  }
}