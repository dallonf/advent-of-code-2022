local inspect = require("scripts.vendor.inspect")

local pairs = {}

function ProcessEvent(event)
  if event.type == "AnalyzePair" then
    table.insert(pairs, { value = event.value, intersections = {}, container = nil })
  elseif event.type == "IntersectionFound" then
    pairs[#pairs].intersections[event.value.position] = true
  elseif event.type == "ContainsOther" then
    local container = nil
    if event.value.which == 0 then
      container = 2
    else
      container = 1
    end
    pairs[#pairs].container = container
  end
end

local function draw_range(ctx, range, x_cursor, y_cursor, intersections, opts)
  for i = range.start, range["end"] do
    local x = x_cursor + (i - 1) * 32
    if x > ctx.width then
      break
    end
    ctx.text(i, x_cursor + (i - 1) * 32, y_cursor, {
      color = intersections[i] and "red" or opts.base_color
    })
  end
end

local start_y = 0

function Draw(ctx)
  local y_cursor = start_y

  local onscreen = {}
  for _, pair in ipairs(pairs) do
    local max = 38
    if pair.value[1].start <= max and pair.value[1]["end"] <= max and
      pair.value[2].start <= max and pair.value[2]["end"] <= max then
      table.insert(onscreen, pair)
    end
  end

  for _, pair in ipairs(onscreen) do
    if y_cursor + (16 + 48) < 0 then
      y_cursor = y_cursor + 16 + 48
      goto continue
    end
    if y_cursor > ctx.height then
      break
    end
    ctx.text(pair.value[1].start .. "-" .. pair.value[1]["end"], 8, y_cursor)
    draw_range(ctx, pair.value[1], 128, y_cursor, pair.intersections, {
      base_color = pair.container == 1 and "green" or "black"
    })
    y_cursor = y_cursor + 16
    ctx.text(pair.value[2].start .. "-" .. pair.value[2]["end"], 8, y_cursor)
    draw_range(ctx, pair.value[2], 128, y_cursor, pair.intersections, {
      base_color = pair.container == 2 and "green" or "black"
    })
    y_cursor = y_cursor + 48
    ::continue::
  end

  start_y = start_y - 1
end
