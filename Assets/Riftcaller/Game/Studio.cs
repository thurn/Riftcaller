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

using Riftcaller.Utils;
using UnityEngine;

namespace Riftcaller.Game
{
  public sealed class Studio : MonoBehaviour
  {
    [SerializeField] Camera _camera = null!;
    [SerializeField] Transform _subjectPosition = null!;
    [SerializeField] GameObject? _subject;

    bool _initialized;
    int _studioLayer;
    public int StudioNumber { get; private set; }
    public RenderTexture RenderTexture { get; private set; } = null!;

    public const string TextureAddress = "Textures/StudioRenderTexture";

    public void Initialize(int studioNumber)
    {
      Errors.CheckState(!_initialized, "Already initialized!");
      name = $"Studio {studioNumber}";
      StudioNumber = studioNumber;
      _studioLayer = LayerMask.NameToLayer("Studio");
      RenderTexture = new RenderTexture(1024, 1024, 16, RenderTextureFormat.Default);
      _camera.targetTexture = RenderTexture;
      _initialized = true;
    }

    public string ClassNameTag() => $"sd_StudioDisplay{StudioNumber}"; 

    void Start()
    {
      _studioLayer = LayerMask.NameToLayer("Studio");      
    }

    public void ClearSubject()
    {
      if (!Application.isPlaying)
      {
        LogUtils.LogError("Not Playing");
        return;
      }

      if (_subject)
      {
        if (Application.isPlaying)
        {
          Destroy(_subject);
        }
      }
    }

    public void SetSubject(GameObject subject)
    {
      subject.transform.SetParent(_subjectPosition);
      subject.transform.localPosition = Vector3.zero;
      subject.transform.localRotation = Quaternion.identity;
      foreach (var t in subject.GetComponentsInChildren<Transform>(true))
      {
        t.gameObject.layer = _studioLayer;
      }      
      _subject = subject;
    }
  }
}