local parts = {}

function ProcessEvent(event)
  if event.type == "NewPart" then
    table.insert(parts, {
      initial_mass = event.value.mass,
      additional = {},
    })
  elseif event.type == "AdditionalFuel" then
    local last_part = parts[#parts]
    table.insert(last_part.additional, event.value.mass)
  end
end

local BAR_HEIGHT = 64
local TEXT_HEIGHT = 24
local ROW_HEIGHT = BAR_HEIGHT + TEXT_HEIGHT
local MASS_PER_PIXEL = 0.004

function Draw(ctx)
  for i, part in ipairs(parts) do
    local offset = i - 1
    local y_offset = offset * ROW_HEIGHT
    local mass = math.tointeger(part.initial_mass)
    ctx.rectangle_fill(0, y_offset + TEXT_HEIGHT, mass * MASS_PER_PIXEL, BAR_HEIGHT, "red")
    local x_cursor = mass * MASS_PER_PIXEL
    local header = "" .. mass
    local total = mass
    for i_additional, additional in ipairs(part.additional) do
      local color = i_additional % 2 == 0 and "red" or "green"
      local width = additional * MASS_PER_PIXEL
      ctx.rectangle_fill(x_cursor, y_offset + TEXT_HEIGHT, width, BAR_HEIGHT, color)
      x_cursor = x_cursor + width
      header = header .. " + " .. math.tointeger(additional)
      total = total + additional
    end
    if #part.additional > 0 then
      header = header .. " = " .. math.tointeger(total)
    end
    ctx.text(header, 8, y_offset, { size = TEXT_HEIGHT - 2 })
  end
end
