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

using System.Collections;
using System.Linq;
using DG.Tweening;
using Spelldawn.Assets;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.AddressableAssets;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class RewardDisplay : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Transform _cardSpawnPosition = null!;
    [SerializeField] Transform _popUpPosition = null!;
    [SerializeField] ObjectDisplay _rewardBrowser = null!;
    [SerializeField] AssetReferenceGameObject _appearEffectReference = null!;
    [SerializeField] Transform _appearEffectPosition = null!;
    [SerializeField] GameObject _appearLight = null!;
    [SerializeField] AssetReferenceGameObject _chestReference = null!;
    [SerializeField] float _duration;
    [SerializeField] AssetReference _buildUpSoundReference = null!;
    [SerializeField] float _openDelay;
    [SerializeField] AudioSource _audioSource = null!;
    [SerializeField] AssetReferenceGameObject _openEffectReference = null!;
    [SerializeField] Transform _openEffectPosition = null!;
    [SerializeField] AssetReference _openSoundReference = null!;
    [SerializeField] bool _canBeOpened;
    [SerializeField] RewardChest _chestPlaceholderPrefab = null!;
    
    Animator? _animator;
    DisplayRewardsCommand? _currentCommand;
    RewardChest? _rewardChest;

    static readonly int Open = Animator.StringToHash("Open");
    
    public void OnOpened()
    {
      _rewardChest!.SetGlowEnabled(false);
      StartCoroutine(_registry.AssetPoolService.CreateFromReference(_openEffectReference, _openEffectPosition.position,
        onCreate: result => result.transform.localScale = _openEffectPosition.localScale));
      StartCoroutine(AssetUtil.PlayReferenceOneShot(_audioSource, _openSoundReference));
      if (_currentCommand != null)
      {
        StartCoroutine(DisplayCards(_currentCommand));
      }
    }

    IEnumerator DisplayCards(DisplayRewardsCommand command)
    {
      var cards = command.Rewards
        .Select(c =>
          _registry.CardService.CreateCard(c, GameContext.RewardBrowser, animate: false))
        .ToList();
      for (var i = 0; i < cards.Count; ++i)
      {
        var card = cards[i];
        card.transform.position = _cardSpawnPosition.position + new Vector3(i * 0.05f, 0, 0);
        card.transform.localScale = Vector3.one * 0.05f;
      }

      foreach (var card in cards)
      {
        yield return TweenUtils.Sequence("PopUpReward")
          .Insert(0, card.transform.DOMove(_popUpPosition.position, 0.3f))
          .Insert(0, card.transform.DOScale(0.1f * Vector3.one, 0.3f))
          .SetEase(Ease.InExpo)
          .WaitForCompletion();
        yield return _rewardBrowser.AddObject(card);
      }
    }

    IEnumerator OnMouseUpAsButton()
    {
      if (_canBeOpened)
      {
        _canBeOpened = false;
        _rewardChest!.SetGlowEnabled(true);
        StartCoroutine(AssetUtil.PlayReferenceOneShot(_audioSource, _buildUpSoundReference));
        yield return new WaitForSecondsRealtime(_openDelay);
        if (_animator)
        {
          _animator!.SetTrigger(Open);
        }
      }
    }

    public IEnumerator HandleDisplayRewards(DisplayRewardsCommand command)
    {
      gameObject.SetActive(true);
      yield return new WaitForSeconds(0.5f);
      StartCoroutine(
        _registry.AssetPoolService.CreateFromReference(_appearEffectReference, _appearEffectPosition.position,
          onCreate: result => result.transform.localScale = _appearEffectPosition.localScale));
      _appearLight.SetActive(true);
      yield return new WaitForSeconds(_duration);

      if (UseProductionAssets.ShouldUseProductionAssets)
      {
        StartCoroutine(_registry.AssetPoolService.CreateFromReference(_chestReference, transform.position,
          onCreate: result =>
          {
            result.transform.localRotation = transform.localRotation;
            _animator = ComponentUtils.GetComponent<Animator>(result);
            _rewardChest = ComponentUtils.GetComponent<RewardChest>(result);
            _rewardChest.RewardDisplay = this;
          }));        
      }
      else
      {
        _rewardChest = ComponentUtils.Instantiate(_chestPlaceholderPrefab);
        _rewardChest.transform.position = transform.position;
        _rewardChest.transform.localRotation = transform.localRotation;
        _rewardChest.RewardDisplay = this;
      }

      _canBeOpened = true;
      _currentCommand = command;
    }
  }
}