# Mock only Windows package discovery and config reads; run the real scanner.
$fixtures = @(
    @{
        Name = 'Gdk'
        Applications = '<Application Id="codShip"><VisualElements DisplayName="Modern Warfare &#174; 3" /></Application>'
        Config = '<Game><ExecutableList><Executable Id="codShip" /></ExecutableList></Game>'
    },
    @{
        Name = 'Legacy'
        Title = 'Legacy title'
        Applications = '<Application Id="Helper" /><Application Id="Game" />'
    },
    @{
        Name = 'Multi'
        Applications = '<Application Id="Helper" /><Application Id="SecondGame"><VisualElements DisplayName="Second title" /></Application><Application Id="DevTool" /><Application Id="HiddenGame"><VisualElements AppListEntry="none" /></Application>'
        Config = '<Game><ExecutableList><Executable Id="SecondGame" /><Executable Id="DevTool" IsDevOnly="true" /><Executable Id="HiddenGame" /></ExecutableList></Game>'
    },
    @{
        Name = 'BrokenConfig'
        Title = 'Fallback title'
        Applications = '<Application Id="Game"><VisualElements DisplayName="ms-resource:Title" /></Application>'
        Config = 'not valid XML'
    },
    @{
        Name = 'Ordinary'
        Applications = '<Application Id="App"><VisualElements DisplayName="Normal app" /></Application>'
    },
    @{
        Name = 'Dlc'
        Config = '<Game />'
    },
    @{
        Name = 'Framework'
        IsFramework = $true
        Applications = '<Application Id="Game" />'
    },
    @{
        Name = 'Resources'
        IsResourcePackage = $true
        Applications = '<Application Id="Game" />'
    },
    @{
        Name = 'BrokenManifest'
        Broken = $true
    }
)

function Get-AppxPackage {
    foreach ($fixture in $fixtures) {
        [PSCustomObject]@{
            Name = $fixture.Name
            PackageFamilyName = 'Fixture.' + $fixture.Name + '_abc'
            InstallLocation = 'C:\Fixture\' + $fixture.Name
            IsFramework = [bool]$fixture.IsFramework
            IsResourcePackage = [bool]$fixture.IsResourcePackage
        }
    }
}

function Get-AppxPackageManifest {
    param($Package, $ErrorAction)
    $fixture = $fixtures | Where-Object Name -eq $Package.Name
    if ($fixture.Broken) { throw 'Unreadable manifest' }
    $title = if ($fixture.Title) { $fixture.Title } else { $fixture.Name }
    [xml]('<Package><Properties><DisplayName>' + $title + '</DisplayName></Properties><Applications>' + $fixture.Applications + '</Applications></Package>')
}

function Test-Path {
    param($LiteralPath)
    $name = Split-Path (Split-Path $LiteralPath) -Leaf
    $fixture = $fixtures | Where-Object Name -eq $name
    [bool]$fixture.Config
}

function Get-Content {
    param($LiteralPath, [switch]$Raw, $ErrorAction)
    $name = Split-Path (Split-Path $LiteralPath) -Leaf
    ($fixtures | Where-Object Name -eq $name).Config
}
