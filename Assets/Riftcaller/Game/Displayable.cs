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

using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.Rendering;

#nullable enable

namespace Riftcaller.Game
{
  public abstract class Displayable : MonoBehaviour
  {
    [Header("Displayable")]
    [SerializeField] GameContext _gameContext;

    [SerializeField] SortingGroup? _sortingGroup;

    IObjectDisplay? _parent;    

    /// <summary>Provided by the server, used to order items within a display.</summary>
    public uint SortingKey { get; set; }

    /// <summary>Tie-breaker key in the case of sorting key ties.</summary>
    public uint SortingSubkey { get; set; }

    public IObjectDisplay? Parent
    {
      get => _parent;
      set => _parent = value;
    }

    public virtual bool IsContainer() => false;

    public virtual float DefaultScale => 1.0f;

    protected void Start()
    {
      if (_sortingGroup && _gameContext != GameContext.Unspecified)
      {
        SortingOrder.Create(_gameContext, (int)SortingKey, (int)SortingSubkey).ApplyTo(_sortingGroup!);
      }

      OnStart();
    }

    /// <summary>
    /// Should return true if this game object can currently handle a MouseDown or MouseHoverStart event.
    /// </summary>
    public virtual bool CanHandleMouseEvents() => false;

    /// <summary>
    /// Invoked on mouse down. Will only be invoked if <see cref="CanHandleMouseEvents"/>
    /// returns true and this is the topmost object hit by the on click raycast.
    /// </summary>
    public virtual void MouseDown()
    {
    }

    /// <summary>
    /// Sent every frame while the mouse button is held down to objects which received <see cref="MouseDown"/>.
    /// </summary>
    public virtual void MouseDrag()
    {
    }

    /// <summary>
    /// Sent on *any* mouse up event, anywhere on screen, to objects which received a <see cref="MouseDown"/>.
    /// </summary>
    public virtual void MouseUp()
    {
    }

    /// <summary>
    /// Invoked on mouse hover start. Will only be invoked if <see cref="CanHandleMouseEvents"/>
    /// returns true and this is the topmost object hit by the on click raycast.
    /// </summary>    
    public virtual void MouseHoverStart()
    {
    }

    public virtual void MouseHover()
    {
    }

    public virtual void MouseHoverEnd()
    {
    }

    protected virtual void OnStart()
    {
    }

    /// <summary>Called on a child container when the parent is repositioned.</summary>
    public virtual void OnUpdateParentContainer()
    {
    }

    public GameContext GameContext => Errors.CheckNotDefault(HasGameContext ? _gameContext : DefaultGameContext());

    public bool HasGameContext => _gameContext != GameContext.Unspecified;

    protected virtual GameContext DefaultGameContext() => GameContext.Unspecified;

    public void SetGameContext(GameContext gameContext)
    {
      Errors.CheckNotDefault(gameContext);

      if (_sortingGroup)
      {
        SortingOrder.Create(gameContext, (int)SortingKey, (int)SortingSubkey).ApplyTo(_sortingGroup!);
      }

      if (_gameContext != gameContext)
      {
        var oldContext = _gameContext;
        _gameContext = gameContext;
        OnSetGameContext(oldContext, gameContext);
      }
    }

    protected abstract void OnSetGameContext(GameContext oldContext, GameContext newContext);
  }
}