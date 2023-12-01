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
using DG.Tweening;
using Riftcaller.Protos;
using Riftcaller.Services;
using Riftcaller.Utils;
using TMPro;
using UnityEngine;
using UnityEngine.AddressableAssets;

#nullable enable

namespace Riftcaller.Game
{
  public sealed class GameMessage : MonoBehaviour
  {
    [Serializable]
    public class MessageContent
    {
      [SerializeField] AssetReferenceGameObject _effectReference = null!;
      public AssetReferenceGameObject EffectReference => _effectReference;

      [SerializeField] TextMeshPro _text = null!;
      public TextMeshPro Text => _text;
    }

    [SerializeField] Registry _registry = null!;
    [SerializeField] Transform _top = null!;
    [SerializeField] MessageContent _dawn = null!;
    [SerializeField] MessageContent _dusk = null!;
    [SerializeField] MessageContent _victory = null!;
    [SerializeField] MessageContent _defeat = null!;
    GameObject? _currentEffect;

    public IEnumerator Show(DisplayGameMessageCommand command)
    {
      switch (command.MessageType)
      {
        case GameMessageType.Dawn:
          _registry.StaticAssets.PlayDawnSound();
          break;
        case GameMessageType.Dusk:
          _registry.StaticAssets.PlayDuskSound();
          break;
        case GameMessageType.Victory:
          _registry.MusicService.SetMusicState(MusicState.Silent);
          _registry.StaticAssets.PlayVictorySound();
          break;
        case GameMessageType.Defeat:
          _registry.MusicService.SetMusicState(MusicState.Silent);
          _registry.StaticAssets.PlayDefeatSound();
          break;
      }

      return command.MessageType switch
      {
        GameMessageType.Dawn => ShowContent(_dawn, 1.75f, moveToTop: false),
        GameMessageType.Dusk => ShowContent(_dusk, 1.75f, moveToTop: false),
        GameMessageType.Victory => ShowContent(_victory, 2f, moveToTop: true),
        GameMessageType.Defeat => ShowContent(_defeat, 2f, moveToTop: true),
        _ => CollectionUtils.Yield()
      };
    }

    IEnumerator ShowContent(MessageContent content, float durationSeconds, bool moveToTop)
    {
      content.Text.transform.position = transform.position;
      yield return _registry.AssetPoolService.CreateFromReference(
        content.EffectReference, 
        transform.position,
        onCreate: result =>
        {
          _currentEffect = result;
        }
        );
      content.Text.gameObject.SetActive(true);
      content.Text.alpha = 0f;
      yield return DOTween
        .To(() => content.Text.alpha, x => content.Text.alpha = x, endValue: 1f, 0.2f)
        .WaitForCompletion();
      yield return new WaitForSeconds(durationSeconds);

      if (moveToTop)
      {
        _registry.InterfaceOverlay.ForceEnable();
        var sequence = TweenUtils.Sequence("MoveToTop")
          .Insert(0, content.Text.transform.DOMove(_top.position, 0.3f));
        if (_currentEffect)
        {
          sequence.Insert(0, _currentEffect!.transform.DOMove(_top.position, 0.3f));
        }
        yield return sequence.WaitForCompletion();
      }
      else
      {
        yield return DOTween
          .To(() => content.Text.alpha, x => content.Text.alpha = x, endValue: 0f, 0.2f)
          .WaitForCompletion();
        content.Text.gameObject.SetActive(false);
      }
    }
  }
}