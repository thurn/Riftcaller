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

using System.Collections;
using UnityEngine;

#nullable enable

namespace Riftcaller.Game
{
  [DisallowMultipleComponent]
  public sealed class TimedEffect : MonoBehaviour
  {
    [SerializeField] float _duration;
    [SerializeField] bool _looping;

    void OnEnable()
    {
      if (!_looping)
      {
        StartCoroutine(DisableAsync(_duration));
      }
    }

    public void SetGameContext(GameContext gameContext)
    {
      foreach (var ps in GetComponentsInChildren<ParticleSystem>())
      {
        SortingOrder.Create(gameContext).ApplyTo(ps.GetComponent<Renderer>());
      }
    }

    public void SetStartColor(Color color)
    {
      foreach (var ps in GetComponentsInChildren<ParticleSystem>())
      {
        var main = ps.main;
        var current = main.startColor.color;
        Color.RGBToHSV(current, out _, out var s, out var v);
        if (s > 0.2f && v > 0.2f)
        {
          main.startColor = new Color(color.r, color.g, color.b, current.a);
        }
      }
    }

    void OnValidate()
    {
      _duration = 0.0f;

      foreach (var system in GetComponentsInChildren<ParticleSystem>())
      {
        var main = system.main;
        _duration = Mathf.Max(_duration, main.duration + main.startLifetime.constantMax);
      }

      foreach (var audioSource in GetComponentsInChildren<AudioSource>())
      {
        if (audioSource.clip != null)
        {
          _duration = Mathf.Max(_duration, audioSource.clip.length);
        }
      }

      foreach (var ps in GetComponentsInChildren<ParticleSystem>())
      {
        SortingOrder.Create(GameContext.Effects).ApplyTo(ps.GetComponent<Renderer>());
      }
    }

    IEnumerator DisableAsync(float duration)
    {
      // Add a little extra time for safety
      yield return new WaitForSeconds(duration + 0.5f);
      gameObject.SetActive(value: false);
    }
  }
}