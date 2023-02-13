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
using System.Collections;
using System.Linq;
using Google.Protobuf.WellKnownTypes;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Tests;
using Spelldawn.World;
using Unity.Services.Core;
using UnityEngine;
using UnityEngine.Serialization;
using Screen = UnityEngine.Device.Screen;

#nullable enable

namespace Spelldawn.Services
{
  public enum GlobalGameMode
  {
    Default,
    ScreenshotTest
  }

  public sealed class Registry : MonoBehaviour
  {
    public GlobalGameMode GlobalGameMode { get; set; } = GlobalGameMode.Default;

    [SerializeField] Camera _mainCamera = null!;
    public Camera MainCamera => _mainCamera;

    [SerializeField] Reporter _logViewer = null!;
    public Reporter LogViewer => _logViewer;

    [SerializeField] AudioSource _mainAudioSource = null!;
    public AudioSource MainAudioSource => _mainAudioSource;

    [SerializeField] GameService _gameService = null!;
    public GameService GameService => _gameService;

    [SerializeField] AssetService _assetService = null!;
    public AssetService AssetService => _assetService;

    [SerializeField] AssetPoolService _assetPoolService = null!;
    public AssetPoolService AssetPoolService => _assetPoolService;

    [SerializeField] ActionService _actionService = null!;
    public ActionService ActionService => _actionService;
    
    [SerializeField] CapabilityService _capabilityService = null!;
    public CapabilityService CapabilityService => _capabilityService;

    [SerializeField] ObjectPositionService _objectPositionService = null!;
    public ObjectPositionService ObjectPositionService => _objectPositionService;

    [SerializeField] CardService _cardService = null!;
    public CardService CardService => _cardService;

    [SerializeField] CommandService _commandService = null!;
    public CommandService CommandService => _commandService;

    [SerializeField] DocumentService _documentService = null!;
    public DocumentService DocumentService => _documentService;

    [SerializeField] InputService _inputService = null!;
    public InputService InputService => _inputService;

    [SerializeField] MusicService _musicService = null!;
    public MusicService MusicService => _musicService;

    [SerializeField] ArrowService _arrowService = null!;
    public ArrowService ArrowService => _arrowService;

    public ArenaService ArenaService => _arenaService;
    [SerializeField] ArenaService _arenaService = null!;

    [SerializeField] RaidService _raidService = null!;
    public RaidService RaidService => _raidService;
    
    [SerializeField] StackObjectDisplay _offscreenCards = null!;
    public StackObjectDisplay OffscreenCards => _offscreenCards;

    [SerializeField] CurveObjectDisplay _cardStaging = null!;
    public CurveObjectDisplay CardStaging => _cardStaging;
    
    [SerializeField] CurveObjectDisplay _revealedCardsBrowserSmall = null!;
    public CurveObjectDisplay RevealedCardsBrowserSmall => _revealedCardsBrowserSmall;
    
    [SerializeField] CurveObjectDisplay _revealedCardsBrowserLarge = null!;
    public CurveObjectDisplay RevealedCardsBrowserLarge => _revealedCardsBrowserLarge;    

    [SerializeField] CardBrowser _cardBrowser = null!;
    public CardBrowser CardBrowser => _cardBrowser;

    [SerializeField] LongPressCardBrowser _longPressBrowser = null!;
    public LongPressCardBrowser LongPressCardBrowser => _longPressBrowser;

    [SerializeField] GameMessage _gameMessage = null!;
    public GameMessage GameMessage => _gameMessage;

    [SerializeField] BackgroundOverlay _interfaceOverlay = null!;
    public BackgroundOverlay InterfaceOverlay => _interfaceOverlay;

    [SerializeField] BackgroundOverlay _raidOverlay = null!;
    public BackgroundOverlay RaidOverlay => _raidOverlay;
    
    [SerializeField] BackgroundOverlay _longPressOverlay = null!;
    public BackgroundOverlay LongPressOverlay => _longPressOverlay;    

    [SerializeField] StaticAssets _staticAssets = null!;
    public StaticAssets StaticAssets => _staticAssets;

    [FormerlySerializedAs("_rewardChest")] [SerializeField] RewardDisplay _rewardDisplay = null!;
    public RewardDisplay RewardDisplay => _rewardDisplay;

    [SerializeField] CurveObjectDisplay _userHand = null!;
    [SerializeField] CurveObjectDisplay _opponentHand = null!;

    public CurveObjectDisplay HandForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userHand : _opponentHand;

    [SerializeField] ObjectDisplay _userDeckPosition = null!;
    [SerializeField] ObjectDisplay _opponentDeckPosition = null!;

    public ObjectDisplay DeckPositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDeckPosition : _opponentDeckPosition;

    [SerializeField] Deck _userDeck = null!;
    [SerializeField] Deck _opponentDeck = null!;

    public Deck DeckForPlayer(PlayerName playerName) => playerName == PlayerName.User ? _userDeck : _opponentDeck;

    [SerializeField] ObjectDisplay _userDiscardPilePosition = null!;
    [SerializeField] ObjectDisplay _opponentDiscardPilePosition = null!;

    public ObjectDisplay DiscardPilePositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDiscardPilePosition : _opponentDiscardPilePosition;

    [SerializeField] DiscardPile _userDiscardPile = null!;
    [SerializeField] DiscardPile _opponentDiscardPile = null!;

    public DiscardPile DiscardPileForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDiscardPile : _opponentDiscardPile;

    [SerializeField] Transform _cardStagingArea = null!;
    public Transform CardStagingArea => _cardStagingArea;

    [SerializeField] ManaDisplay _userManaDisplay = null!;
    [SerializeField] ManaDisplay _opponentManaDisplay = null!;

    public ManaDisplay ManaDisplayForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userManaDisplay : _opponentManaDisplay;

    [SerializeField] ActionDisplay _userActionDisplay = null!;
    [SerializeField] ActionDisplay _opponentActionDisplay = null!;

    public ActionDisplay ActionDisplayForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userActionDisplay : _opponentActionDisplay;

    [SerializeField] ObjectDisplay _userLeaderCardPosition = null!;
    [SerializeField] ObjectDisplay _opponentLeaderCardPosition = null!;

    public ObjectDisplay LeaderCardPositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userLeaderCardPosition : _opponentLeaderCardPosition;

    [SerializeField] LeaderCard _userLeaderCard = null!;
    [SerializeField] LeaderCard _opponentLeaderCard = null!;

    public LeaderCard LeaderCardForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userLeaderCard : _opponentLeaderCard;

    [SerializeField] GameObject? _userActiveLight;
    [SerializeField] GameObject? _opponentActiveLight;

    public GameObject? ActiveLightForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userActiveLight : _opponentActiveLight;

    [SerializeField] GameObject _graphy = null!;
    public GameObject Graphy => _graphy;

    [SerializeField] Studio _studio = null!;
    public Studio Studio => _studio;

    [SerializeField] SettingsService _settingsService = null!;
    public SettingsService SettingsService => _settingsService;

    public ScreenshotTestService? ScreenshotTests { get; private set; }

    [SerializeField] WorldMap _worldMap = null!;
    public WorldMap WorldMap => _worldMap;

    [SerializeField] WorldCharacterService _characterService = null!;
    public WorldCharacterService CharacterService => _characterService;

    [SerializeField] UpdateInterfaceService _updateInterfaceService = null!;
    public UpdateInterfaceService UpdateInterfaceService => _updateInterfaceService;

    [SerializeField] TutorialService _tutorialService = null!;
    public TutorialService TutorialService => _tutorialService;

    [SerializeField] AnalyticsService _analyticsService = null!;
    public AnalyticsService AnalyticsService => _analyticsService;

    [SerializeField] UserReportingScript _userReportingScript = null!;
    public UserReportingScript UserReportingScript => _userReportingScript;

    [SerializeField] StudioManager _studioManager = null!;
    public StudioManager StudioManager => _studioManager;

    async void Awake()
    {
      try
      {
        await UnityServices.InitializeAsync();
      }
      catch (Exception e)
      {
        Debug.LogException(e);
      }
    }    
    
    IEnumerator Start()
    {
      Application.targetFrameRate = 60;
      var runTests = false;
      var testService = FindObjectOfType<ScreenshotTestService>();

      if (GlobalGameMode == GlobalGameMode.ScreenshotTest ||
          testService ||
          Environment.GetCommandLineArgs().Any(arg => arg.Contains("test")))
      {
        GlobalGameMode = GlobalGameMode.ScreenshotTest;
        ScreenshotTests = ScreenshotTestService.Initialize(this, out runTests);
      }
      
      DocumentService.Initialize();
      MusicService.Initialize(GlobalGameMode);
      GameService.Initialize(GlobalGameMode);

      if (ArenaService != null)
      {
        yield return ArenaService.Initialize();
      }

      if (runTests)
      {
        ScreenshotTests!.RunTests();
      }
    }

    void OnEnable()
    {
      Application.logMessageReceived += HandleLog;
    }
    
    void OnDisable()
    {
      Application.logMessageReceived -= HandleLog;
    }    

    void HandleLog(string condition, string stacktrace, LogType type)
    {
      if (type is LogType.Error or LogType.Exception)
      {
        if (!condition.Contains("RpcException") && !condition.Contains("[Error]"))
        {
          StartCoroutine(CommandService.HandleCommands(new GameCommand
          {
            Debug = new ClientDebugCommand
            {
              ShowLogs = new Empty()
            }
          }));          
        }
      }
    }
  }
}