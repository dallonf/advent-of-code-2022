local parts = {}

function ProcessEvent(event)
  if event.type == "NewPart" then
    table.insert(parts, event.value.mass)
  end
end

local BAR_HEIGHT = 64 + 48

function Draw(ctx)
  for i, mass in ipairs(parts) do
    local offset = i - 1
    ctx.draw_rectangle(0, offset * BAR_HEIGHT, mass * 0.004, BAR_HEIGHT)
  end

  return "hi"
end
