# General changes

## Transports

**OLD:** Transport units (APC, T Copter, Lander, Cruiser, Carrier, and Black Boat)
can only load / unload once per turn.

**NEW:** Transport units are allowed to load / unload an unlimited number of times per turn.
In AWBW this is called "transport boosting".

Without transport boosting there is very little reason to make transports, and even with
boosting transports are still pretty bad.

Transport boosting makes transports more fun to use, it increases the amount of tactic depth
and positioning.


## Bad luck COs

Some COs (Flak, Jugger, and Sonja) have bad luck.

**OLD:** Advance Wars rolls 2 numbers, 1 number for the good luck, and 1 number for the bad luck, and then subtracts them.

This means that good luck tends to cancel out bad luck, and bad luck tends to cancel out good luck, so the
end result is that Flak / Jugger / Sonja tend to have very average luck.

But the entire point of COs like Flak / Jugger is that they're supposed to have wildly varying luck, not
average luck!

**NEW:** So Rusted Battalions works differently. Instead of rolling 2 numbers and cancelling them out, Rusted
Battalions just rolls 1 number to determine the luck value.

So for example, Flak has +15% good luck and -10% bad luck. Rusted Battalions will roll a number from
-10 to +15 and that will be the luck for that attack.

This means that the bad luck COs have more variability with their attacks, even though they have the same
luck stats from Advance Wars.


## HP display

**OLD:** Advance Wars does not display the actual HP of the unit, instead it displays the rounded HP of the unit (from 1 to 10).

**NEW:** Rusted Battalions displays the actual HP of the unit (from 1 to 100).


## HP rounding

In many situations, Advance Wars uses the rounded HP for calculations. Rusted Battalions uses the non-rounded HP.

**OLD:** Damage and luck are scaled based on the rounded HP of the attacking unit.

**OLD:** Terrain star bonus is scaled based on the rounded HP of the defending unit.

**OLD:** Capturing properties uses the rounded HP of the infantry.

**OLD:** When joining units, if the HP is above 100 you gain extra funds based on the rounded HP of the unit.

**OLD:** Repairs are based on the rounded HP of the unit, which means it's possible for a unit to be repaired +29 HP.

**OLD:** The cost of repairs is based on the rounded HP of the unit.

**NEW:** In Rusted Battalions, the above mentioned things use the non-rounded HP of units.


## HP of structures

**OLD:** In Advance Wars, structures like pipeseams have 99 HP.

**NEW:** In Rusted Battalions, structures have 100 HP.


## Transported units

**OLD:** In Advance Wars, if a transport unit is killed while it is holding other units, the held units do not add
to the power meter.

**NEW:** In Rusted Battalions, the held units do add to the power meter.


## Joining units

**OLD:** In Advance Wars, a 100 HP unit can join with a <100 HP unit, but a <100 HP unit cannot join with a 100 HP unit.

**NEW:** In Rusted Battalions, units can always join, regardless of their HP.


## Repairs

**OLD:** If a unit is at 89 HP or less and you cannot afford the full +20 HP repairs, Advance Wars will not
repair the unit.

**NEW:** In Rusted Battalions, it repairs as much as you can afford. So if you can only afford 7 HP of repairs, then
it will repair the unit with +7 HP.


## Resupply

**OLD:** In Advance Wars, if an air / sea unit is at 0 fuel, it is guaranteed to die, even if it's next to an APC.
This is because fuel deductions are calculated before resupply.

**NEW:** In Rusted Battalions, resupply is calculated first, and so the air / sea unit will live.


## COs with different unit costs

Some COs like Kanbei, Colin, and Hachi are able to build units with a different cost than normal.

This affects the repair cost, join unit cost, and how much CO power meter is gained.

**OLD:** In Advance Wars, the repair cost / join unit cost / power meter generation are based on the current cost of the unit,
which means that when Hachi uses his (Super) CO power, it reduces the cost of all units by 50%, including units built on previous turns.

**NEW:** In Rusted Battalions, the cost is based on the cost of the unit when the unit was built, which means that when Hachi uses his (Super) CO power, it does not affect the cost for pre-existing units, it only affects newly built units.

----

# CO changes

## Koal

* **NEW:** Bridges now count as roads, which means that Koal benefits from them.
