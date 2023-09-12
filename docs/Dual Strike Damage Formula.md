This damage formula might not be 100% accurate, but it should be very close.

```
bonus damage = CO bonuses + 10% per comtower + 10% CO power bonus
terrain defense = HP * (10% for each terrain star)
defense = terrain defense + CO bonuses
luck = truncate(0-9% by default + CO bonuses)
damage = truncate(min(truncate(base damage * bonus damage * defense), 1%) + luck)
```
