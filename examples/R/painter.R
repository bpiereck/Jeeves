# Draws a histogram of the entire image
#

library("ggplot2")
library("jsonlite")
library("ragg")
library("magick")

image_width <- 40
image_height <- 40

to_greyscale <- function(pixel) {
  ceiling(0.21*pixel[1] + 0.72*pixel[2] + 0.07*pixel[3])
}

mytheme <- theme(
  axis.title.x = element_blank(),
  axis.title.y = element_blank(),
  axis.text.x = element_blank(),
  axis.text.y = element_blank(),
  axis.ticks = element_blank(),
  panel.grid.major = element_blank(),  # Remove major grid lines
  panel.grid.minor = element_blank(),  # Remove minor grid lines
  panel.background = element_blank()   # Remove panel background
)

ws <- websocket::WebSocket$new("wss://rse.pagekite.me", autoConnect = FALSE)
ws$onMessage(function(event) {
  if (is.raw(event$data)) {
    # This is pixel data
    raw <- as.numeric(tail(event$data, -2))  # We don't care about the image dimensions
    pixels <- split(raw, ceiling(seq_along(raw) / 4))
    grey_pixels <- as.numeric(lapply(pixels, to_greyscale))
    pixel_df <- data.frame(brightness = grey_pixels)

    plot <- ggplot(pixel_df, aes(x = brightness)) + geom_histogram(binwidth = 25) + mytheme
    tmp_file <- tempfile(fileext = ".png")
    ragg::agg_png(filename = tmp_file, width = image_width, height = image_height, res = 1, bg = "transparent")
    print(plot)
    dev.off()
    img <- magick::image_read(tmp_file)
    raw_rgba <- as.raw(magick::image_data(img, "rgba"))
    unlink(tmp_file)
    ws$send(raw_rgba)
  } else {
    message <- fromJSON(event$data)
    switch(message$msg,
      "?" = {
        ws$send("{\"msg\": \"?\", \"?\": \"painter\", \"name\": \"James Collier\", \"url\": \"https://james.thecolliers.xyz\"}")
      },
      "size" = {
        image_width <- message$w
        image_height <- message$h
      },
      "p" = {
        # We're being asked for pixels
        # Ask for the entire image so we can compute the histogram
        ws$send("{\"msg\": \"p\"}")
      },
      {
        cat("Unknown message: ", toString(message))
      }
    )
  }
})

ws$connect()

repeat {
  later::run_now(timeout = 1)
}