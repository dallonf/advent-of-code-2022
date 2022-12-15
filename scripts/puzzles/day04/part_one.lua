local inspect = require("scripts.vendor.inspect")

local pairs = {}

function ProcessEvent(event)
  if event.type == "AnalyzePair" then
    table.insert(pairs, { value = event.value })
  elseif event.type == "IntersectionFound" then
    pairs[#pairs].intersection_at = event.value.position
  end
end

local function draw_range(ctx, range, x_cursor, y_cursor, intersection_at)
  for i = range.start, range["end"] do
    ctx.text(i, x_cursor + (i - 1) * 32, y_cursor, {
      color = intersection_at == i and "red" or "black"
    })
  end
end

function Draw(ctx)
  local y_cursor = 8
  for _, pair in ipairs(pairs) do
    ctx.text(pair.value[1].start .. "-" .. pair.value[1]["end"], 8, y_cursor)
    draw_range(ctx, pair.value[1], 128, y_cursor, pair.intersection_at)
    y_cursor = y_cursor + 16
    ctx.text(pair.value[2].start .. "-" .. pair.value[2]["end"], 8, y_cursor)
    draw_range(ctx, pair.value[2], 128, y_cursor, pair.intersection_at)
    y_cursor = y_cursor + 48
  end
end
