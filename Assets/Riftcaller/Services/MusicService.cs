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
using System.Collections.Generic;
using DG.Tweening;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.Serialization;
using Random = UnityEngine.Random;

#nullable enable

namespace Riftcaller.Services
{
  public sealed class MusicService : MonoBehaviour
  {
    [FormerlySerializedAs("_gameplayAudioSource")] [SerializeField] AudioSource _audioSourceA = null!;
    [FormerlySerializedAs("_raidAudioSource")] [SerializeField] AudioSource _audioSourceB = null!;
    
    [SerializeField] MusicState _audioState = MusicState.Unspecified;
    [SerializeField] List<AudioClip> _mainMenuTracks = null!;
    [SerializeField] List<AudioClip> _gameplayTracks = null!;
    [SerializeField] List<AudioClip> _raidTracks = null!;
    AudioSource? _currentAudioSource;

    public void Initialize(GlobalGameMode globalGameMode)
    {
      SyncVolume();
      
      if (globalGameMode == GlobalGameMode.Default)
      {
        SetMusicState(SceneManager.GetActiveScene().name == "Main" ? MusicState.MainMenu : MusicState.Gameplay);
      }
    }

    /// <summary>Sets audio source volume by reading the value of the music volume PlayerPref</summary>
    public void SyncVolume()
    {
      if (PlayerPrefs.HasKey(Preferences.MusicVolume))
      {
        _audioSourceA.volume = PlayerPrefs.GetFloat(Preferences.MusicVolume);
        _audioSourceB.volume = PlayerPrefs.GetFloat(Preferences.MusicVolume);             
      }
      else
      {
        _audioSourceA.volume = 0.25f;
        _audioSourceB.volume = 0.25f;
      }
    }

    public void SetMusicState(MusicState state)
    {
      if (_audioState != state)
      {
        if (_currentAudioSource)
        {
          var source = _currentAudioSource!;
          TweenUtils
            .Sequence("FadeOutAudio")
            .Append(source.DOFade(0, 1.0f))
            .AppendCallback(() => source.Stop());
        }

        _audioState = state;
        if (state == MusicState.Silent)
        {
          _currentAudioSource = null;
          return;
        }

        var track = state switch
        {
          MusicState.Gameplay => _gameplayTracks[Random.Range(0, _gameplayTracks.Count)],
          MusicState.Raid => _raidTracks[Random.Range(0, _raidTracks.Count)],
          MusicState.MainMenu => _mainMenuTracks[Random.Range(0, _raidTracks.Count)],
          _ => throw new ArgumentOutOfRangeException(nameof(state), state, null)
        };

        _currentAudioSource = _currentAudioSource == _audioSourceA ? _audioSourceB : _audioSourceA;
        _currentAudioSource.clip = track;
        _currentAudioSource.volume = 0f;
        _currentAudioSource.DOFade(PlayerPrefs.GetFloat(Preferences.MusicVolume), 1.0f);
        _currentAudioSource.Play();
      }
    }
  }
}