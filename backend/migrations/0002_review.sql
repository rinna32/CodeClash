-- Разбор решения от нейросети после матча. JSON { summary, approach, mistakes, tips }.
-- Храним, чтобы при повторном открытии матча не тратить лимит нейросети заново
ALTER TABLE matches ADD COLUMN review TEXT;
