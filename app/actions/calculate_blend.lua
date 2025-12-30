-- calculate_blend.lua
--
-- Calculates how much E85 to add to reach a target ethanol percentage.
--
-- Inputs (from Store):
--   inputs.current_ethanol_pct - Current ethanol percentage in tank
--   inputs.target_ethanol_pct  - Desired ethanol percentage
--   inputs.current_fuel_liters - Current amount of fuel in tank
--
-- Outputs (to Store):
--   outputs.e85_to_add_liters  - Amount of E85 to add
--   errors.inputs.*            - Validation error messages (if any)
--
-- Formula:
--   E85 is approximately 85% ethanol.
--   To reach target% from current% with V liters of fuel:
--   x = (target - current) * V / (E85_PCT - target)
--   where x is the liters of E85 to add.

app.log("Running calculate_blend...")

-- Clear any previous errors
app.set("errors.inputs.current_ethanol_pct", nil)
app.set("errors.inputs.target_ethanol_pct", nil)
app.set("errors.inputs.current_fuel_liters", nil)

-- Read inputs from store (they come in as strings from TextInput)
local current_str = app.get("inputs.current_ethanol_pct") or ""
local target_str = app.get("inputs.target_ethanol_pct") or ""
local fuel_str = app.get("inputs.current_fuel_liters") or ""

-- Parse inputs
local current = tonumber(current_str)
local target = tonumber(target_str)
local fuel = tonumber(fuel_str)

-- Validate inputs
local has_error = false

if current_str == "" then
    app.set("errors.inputs.current_ethanol_pct", "Required")
    has_error = true
elseif current == nil then
    app.set("errors.inputs.current_ethanol_pct", "Must be a number")
    has_error = true
elseif current < 0 or current > 100 then
    app.set("errors.inputs.current_ethanol_pct", "Must be 0-100")
    has_error = true
end

if target_str == "" then
    app.set("errors.inputs.target_ethanol_pct", "Required")
    has_error = true
elseif target == nil then
    app.set("errors.inputs.target_ethanol_pct", "Must be a number")
    has_error = true
elseif target < 0 or target > 100 then
    app.set("errors.inputs.target_ethanol_pct", "Must be 0-100")
    has_error = true
end

if fuel_str == "" then
    app.set("errors.inputs.current_fuel_liters", "Required")
    has_error = true
elseif fuel == nil then
    app.set("errors.inputs.current_fuel_liters", "Must be a number")
    has_error = true
elseif fuel < 0 then
    app.set("errors.inputs.current_fuel_liters", "Must be positive")
    has_error = true
end

if has_error then
    app.set("outputs.e85_to_add_liters", "---")
    app.log("Validation failed")
    return
end

-- E85 is approximately 85% ethanol
local E85_ETHANOL_PCT = 85

-- Calculate result
local result

if target >= E85_ETHANOL_PCT then
    -- Can't exceed E85 percentage
    app.set("outputs.e85_to_add_liters", "N/A")
    app.set("errors.inputs.target_ethanol_pct", "Cannot exceed 85%")
    app.log("Target exceeds E85 percentage")
elseif target <= current then
    -- Already at or above target
    app.set("outputs.e85_to_add_liters", "0.00")
    app.log("Already at or above target")
elseif fuel <= 0 then
    -- No fuel to blend
    app.set("outputs.e85_to_add_liters", "0.00")
    app.log("No fuel to blend")
else
    -- Calculate: x = (target - current) * fuel / (E85_PCT - target)
    result = (target - current) * fuel / (E85_ETHANOL_PCT - target)
    app.set("outputs.e85_to_add_liters", string.format("%.2f", result))
    app.log(string.format("Calculated: %.2f liters of E85 needed", result))
end

app.log("calculate_blend complete")
