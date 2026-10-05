-- Optional custom banner image for a Package's card — distinct from the existing small square
-- icon (icon_type/icon_name/icon_url). NULL (default, every pre-existing row) means the card
-- keeps rendering its gradient + icon exactly as before — banner_url is purely additive, never
-- required. See frontend/components/shared/BannerPicker.vue.
ALTER TABLE packages ADD COLUMN banner_url TEXT;
