function Get-AlphaDeliveryNotBefore {
    param([Parameter(Mandatory)][string]$Root,[datetimeoffset]$Requested=[datetimeoffset]::MinValue)
    $TogetherSchedulePath=Join-Path $Root 'config/alpha-delivery.json'
    if(Test-Path -LiteralPath $TogetherSchedulePath) {
        $TogetherSchedule=Get-Content -LiteralPath $TogetherSchedulePath -Raw | ConvertFrom-Json
        # ConvertFrom-Json may already return DateTime. Do not stringify it and
        # parse that zone-less display text as local time a second time.
        $TogetherValue=$TogetherSchedule.not_before_utc
        $TogetherScheduled=if($TogetherValue -is [datetime] -or $TogetherValue -is [datetimeoffset]) {
            [datetimeoffset]$TogetherValue
        } else {
            [datetimeoffset]::Parse($TogetherValue,[Globalization.CultureInfo]::InvariantCulture)
        }
        if($TogetherScheduled -gt $Requested){return $TogetherScheduled}
    }
    return $Requested
}

function Assert-AlphaDeliveryWindow {
    param([Parameter(Mandatory)][string]$Root)
    $TogetherNotBefore=Get-AlphaDeliveryNotBefore -Root $Root
    if([datetimeoffset]::UtcNow -lt $TogetherNotBefore){throw 'Public Alpha publication is held until the user-requested delivery day'}
}
