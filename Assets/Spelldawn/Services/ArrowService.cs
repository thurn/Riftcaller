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

using System;
using Spelldawn.Assets;
using Spelldawn.Game;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ArrowService : MonoBehaviour
  {
    public enum Type
    {
      Red,
      Green,
      Blue
    }

    public interface IArrowAnchor
    {
      Vector3 GetPosition();
    }

    public sealed class MouseAnchor : IArrowAnchor
    {
      readonly Registry _registry;
      readonly float _dragStartScreenZ;

      public MouseAnchor(Registry registry, float dragStartScreenZ)
      {
        _registry = registry;
        _dragStartScreenZ = dragStartScreenZ;
      }
      
      public Vector3 GetPosition() =>
        _registry.MainCamera.ScreenToWorldPoint(
          new Vector3(Input.mousePosition.x, Input.mousePosition.y, _dragStartScreenZ));
    }

    public sealed class TransformAnchor : IArrowAnchor
    {
      readonly Transform _anchorTo;
      
      public TransformAnchor(Transform anchorTo)
      {
        _anchorTo = anchorTo;
      }

      public Vector3 GetPosition() => _anchorTo.position;
    }    
    
    public interface IArrowDelegate
    {
      void OnArrowMoved(Vector3 position);

      void OnArrowReleased(Vector3 position);
    }

    [SerializeField] Registry _registry = null!;
    [SerializeField] Arrow _redArrow = null!;
    [SerializeField] Arrow _greenArrow = null!;
    [SerializeField] Arrow _blueArrow = null!;

    [SerializeField] Arrow? _currentArrow;
    //[SerializeField] Vector3 _startPosition;
    //[SerializeField] float _dragStartScreenZ;
    [SerializeField] GameObject _placeholderHeadPrefab = null!;
    [SerializeField] GameObject _placeholderPiecePrefab = null!;
    IArrowDelegate? _delegate;
    IArrowAnchor? _source;
    IArrowAnchor? _target;

    void Start()
    {
      if (!UseProductionAssets.ShouldUseProductionAssets)
      {
        _redArrow.HeadPrefab = _placeholderHeadPrefab;
        _redArrow.PiecePrefab = _placeholderPiecePrefab;
        _greenArrow.HeadPrefab = _placeholderHeadPrefab;
        _greenArrow.PiecePrefab = _placeholderPiecePrefab;
        _blueArrow.HeadPrefab = _placeholderHeadPrefab;
        _blueArrow.PiecePrefab = _placeholderPiecePrefab;        
      }
    }

    public void ShowMouseArrow(Type type, Transform source, IArrowDelegate? arrowDelegate)
    {
      HideArrows();
      _currentArrow = ArrowForType(type);
      _delegate = arrowDelegate;
      var dragStartScreenZ = _registry.MainCamera.WorldToScreenPoint(source.position).z;
      _source = new TransformAnchor(source);
      _target = new MouseAnchor(_registry, dragStartScreenZ);
    }

    public void ShowArrow(Type type, IArrowAnchor source, IArrowAnchor target)
    {
      HideArrows();
      _currentArrow = ArrowForType(type);
      _source = source;
      _target = target;
    }    
    
    void Update()
    {
      if (_currentArrow && _currentArrow != null && _source is {} source && _target is {} target)
      {
        var sourcePosition = source.GetPosition();
        var targetPosition = target.GetPosition();

        if (Input.GetMouseButton(0))
        {
          if (Vector3.Distance(sourcePosition, targetPosition) < 3.0f)
          {
            _currentArrow.gameObject.SetActive(false);
          }
          else
          {
            _currentArrow.gameObject.SetActive(true);
            _currentArrow.Source = sourcePosition;
            _currentArrow.Target = targetPosition;
            _delegate?.OnArrowMoved(targetPosition);
          }
        }
        else
        {
          _currentArrow.gameObject.SetActive(false);
          _currentArrow = null;
          _delegate?.OnArrowReleased(targetPosition);
        }
      }
    }

    public void HideArrows()
    {
      _currentArrow = null;
      _redArrow.gameObject.SetActive(false);
      _greenArrow.gameObject.SetActive(false);
      _blueArrow.gameObject.SetActive(false);
    }

    Arrow ArrowForType(Type type) => type switch
    {
      Type.Red => _redArrow,
      Type.Green => _greenArrow,
      Type.Blue => _blueArrow,
      _ => throw new ArgumentOutOfRangeException(nameof(type), type, null)
    };
  }
}